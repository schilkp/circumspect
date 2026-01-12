pub mod addr2line;
pub mod disasm;

use anyhow::anyhow;
use log::{debug, trace, warn};
use synthetto::{decode_length_delimiter, Message, TracePacket};
use tempfile::NamedTempFile;

use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Seek, Write},
    path::Path,
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Placeholder {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedElement {
    Text(String),
    Placeholder(Placeholder),
}

pub trait Annotater {
    fn accepts_keys(&self) -> Vec<String>;
    fn annotate(&mut self, placeholder: &Placeholder) -> anyhow::Result<Option<String>>;
}

type AnnotatorRef = Rc<RefCell<Box<dyn Annotater>>>;

pub fn annotate(
    input: &Path,
    output: Option<&Path>,
    annotators_list: Vec<Box<dyn Annotater>>,
) -> anyhow::Result<()> {
    // Build map of annotators:
    let mut annotators: HashMap<String, AnnotatorRef> = HashMap::new();
    for annotator in annotators_list {
        let keys = annotator.accepts_keys();
        let annotator = Rc::new(RefCell::new(annotator));
        for key in keys {
            annotators.insert(key, annotator.clone());
        }
    }

    // Decide input/output files:
    let (output_path, temp_file) = match output {
        Some(path) => (path.to_path_buf(), None),
        None => {
            // Create temp file in the same directory as input
            let input_dir = input.parent().unwrap_or(Path::new("."));
            let temp = NamedTempFile::new_in(input_dir)?;
            let temp_path = temp.path().to_path_buf();
            debug!(
                "no output file given - creating temp file {}",
                temp_path.to_string_lossy()
            );
            (temp_path, Some(temp))
        }
    };

    {
        let input_file = File::open(input)?;
        let mut reader = BufReader::new(input_file);
        let mut decode_buffer: Vec<u8> = Vec::with_capacity(128);

        let output_file = File::create(&output_path)?;
        let mut writer = BufWriter::new(output_file);
        let mut encode_buffer: Vec<u8> = Vec::with_capacity(128);

        loop {
            trace!(
                "Reading package @ {}",
                reader.stream_position().unwrap_or(0)
            );
            // Read tag:
            let mut tag: [u8; 1] = [0];
            let tag_bytes_cnt = reader.read(&mut tag)?;
            if tag_bytes_cnt == 0 {
                break; // No more to read.
            }
            if tag[0] != 0x0A {
                return Err(anyhow!(
                    "invalid trace: tag is 0x{:x}, expected 0x0A",
                    tag[0]
                ));
            }

            trace!("  Tag OK.");

            // Read length varint:
            let mut len_num_bytes: usize = 0;
            let mut len_field: [u8; 19] = [0; 19];
            loop {
                let mut byte: [u8; 1] = [0];
                reader.read_exact(&mut byte)?;
                len_field[len_num_bytes] = byte[0];
                len_num_bytes += 1;
                if (byte[0] & 0x80) == 0 {
                    break;
                }
                if len_num_bytes == 19 {
                    return Err(anyhow!("invalid trace: len is un-terminated varint."));
                }
            }
            let len: usize = decode_length_delimiter(&len_field[0..len_num_bytes])?;

            if len == 0 {
                return Err(anyhow!("invalid trace: zero-len package."));
            }

            trace!("  read {} bytes for len field. len: {}", len_num_bytes, len);

            // Read field:
            decode_buffer.resize(len, 0);
            reader.read_exact(&mut decode_buffer)?;

            // Decode TracePacket:
            let packet = TracePacket::decode(&*decode_buffer)?;

            // Apply annotations:
            let transformed_packet = annotate_packet(packet, &annotators)?;

            // Write-back to output:
            writer.write_all(&tag)?;
            if let Some(transformed_packet) = transformed_packet {
                encode_buffer.clear();
                transformed_packet.encode_length_delimited(&mut encode_buffer)?;
                writer.write_all(&encode_buffer)?;
            } else {
                writer.write_all(&len_field[0..len_num_bytes])?;
                writer.write_all(&decode_buffer)?;
            }
        }
    }

    // If we used a temp file, move it to overwrite the input
    if let Some(temp) = temp_file {
        trace!(
            "no output file given - replacing input {} with temp file {}",
            input.to_string_lossy(),
            temp.path().to_string_lossy()
        );
        fs::rename(&output_path, input)?;
    }
    Ok(())
}

fn annotate_packet(
    mut pkt: TracePacket,
    annotators: &HashMap<String, AnnotatorRef>,
) -> anyhow::Result<Option<TracePacket>> {
    let mut did_modify = false;

    if let Some(synthetto::trace_packet::Data::TrackEvent(evt)) = &mut pkt.data {
        if let Some(synthetto::track_event::NameField::Name(name)) = &mut evt.name_field {
            did_modify |= annotate_string(name, annotators)?;
        }
    }

    if let Some(synthetto::trace_packet::Data::TrackDescriptor(evt)) = &mut pkt.data {
        if let Some(description) = &mut evt.description {
            did_modify |= annotate_string(description, annotators)?;
        }

        if let Some(synthetto::track_descriptor::StaticOrDynamicName::Name(name)) =
            &mut evt.static_or_dynamic_name
        {
            did_modify |= annotate_string(name, annotators)?;
        }

        if let Some(proc) = &mut evt.process {
            if let Some(name) = &mut proc.process_name {
                did_modify |= annotate_string(name, annotators)?;
            }
            for cmdline_part in &mut proc.cmdline {
                did_modify |= annotate_string(cmdline_part, annotators)?;
            }
        }

        if let Some(thread) = &mut evt.thread {
            if let Some(name) = &mut thread.thread_name {
                did_modify |= annotate_string(name, annotators)?;
            }
        }
    }

    if did_modify {
        Ok(Some(pkt))
    } else {
        Ok(None)
    }
}

fn annotate_string(
    s: &mut String,
    annotators: &HashMap<String, AnnotatorRef>,
) -> anyhow::Result<bool> {
    let mut did_modify = false;

    let parts_input = parse_string(s);
    let mut parts_processed: Vec<String> = Vec::with_capacity(parts_input.len());

    for part in parts_input {
        match part {
            ParsedElement::Text(s) => parts_processed.push(s),
            ParsedElement::Placeholder(placeholder) => {
                if let Some(annotator) = annotators.get(&placeholder.kind) {
                    if let Some(replacement) = annotator.borrow_mut().annotate(&placeholder)? {
                        did_modify = true;
                        parts_processed.push(replacement);
                    } else {
                        parts_processed.push(placeholder.value);
                    }
                } else {
                    warn!(
                        "Found placeholder '{}' in string '{}' for which no annotator is known.",
                        placeholder.kind, s
                    );
                    parts_processed.push(format!("${}:{}", placeholder.kind, placeholder.value));
                }
            }
        }
    }

    if did_modify {
        *s = parts_processed.join("");
    }

    Ok(did_modify)
}

pub fn parse_string(input: &str) -> Vec<ParsedElement> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current_text = String::new();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Save any accumulated text before the placeholder
            if !current_text.is_empty() {
                result.push(ParsedElement::Text(current_text.clone()));
                current_text.clear();
            }

            // Try to parse placeholder
            if let Some(placeholder) = parse_placeholder(&mut chars) {
                result.push(ParsedElement::Placeholder(placeholder));
            } else {
                // If parsing failed, treat '$' as regular text
                current_text.push('$');
            }
        } else {
            current_text.push(ch);
        }
    }

    // Add any remaining text
    if !current_text.is_empty() {
        result.push(ParsedElement::Text(current_text));
    }

    result
}

fn parse_placeholder(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<Placeholder> {
    let mut kind = String::new();
    let mut value = String::new();
    let mut parsing_kind = true;

    while let Some(&ch) = chars.peek() {
        match ch {
            ':' if parsing_kind => {
                chars.next(); // consume ':'
                parsing_kind = false;
            }
            ' ' | '\t' | '\n' | '\r' | '$' => break,
            _ => {
                chars.next(); // consume the character
                if parsing_kind {
                    kind.push(ch);
                } else {
                    value.push(ch);
                }
            }
        }
    }

    if !kind.is_empty() && !parsing_kind && !value.is_empty() {
        Some(Placeholder { kind, value })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_only_placeholder() {
        let input = "$kind:value";
        let result = parse_string(input);

        dbg!(&result);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            ParsedElement::Placeholder(Placeholder {
                kind: "kind".to_string(),
                value: "value".to_string()
            })
        );
    }

    #[test]
    fn test_parse_mixed_content() {
        let input = "Hello $name:Silvano and welcome to $place:ETH!";
        let result = parse_string(input);

        dbg!(&result);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], ParsedElement::Text("Hello ".to_string()));
        assert_eq!(
            result[1],
            ParsedElement::Placeholder(Placeholder {
                kind: "name".to_string(),
                value: "Silvano".to_string()
            })
        );
        assert_eq!(
            result[2],
            ParsedElement::Text(" and welcome to ".to_string())
        );
        assert_eq!(
            result[3],
            ParsedElement::Placeholder(Placeholder {
                kind: "place".to_string(),
                value: "ETH!".to_string()
            })
        );
    }

    struct CapitalizeAnnotator;

    impl Annotater for CapitalizeAnnotator {
        fn accepts_keys(&self) -> Vec<String> {
            vec!["cap".to_string()]
        }

        fn annotate(&mut self, placeholder: &Placeholder) -> anyhow::Result<Option<String>> {
            if placeholder.kind == "cap" {
                Ok(Some(placeholder.value.to_uppercase()))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_annotate_string_capitalize() {
        let mut annotators: HashMap<String, AnnotatorRef> = HashMap::new();
        let annotator = Rc::new(RefCell::new(
            Box::new(CapitalizeAnnotator) as Box<dyn Annotater>
        ));
        annotators.insert("cap".to_string(), annotator);

        let mut test_string = "Hello, $cap:world !".to_string();
        let result = annotate_string(&mut test_string, &annotators).unwrap();

        assert!(result);
        assert_eq!(test_string, "Hello, WORLD !");
    }

    #[test]
    fn test_annotate_string_no_matching_annotator() {
        let annotators: HashMap<String, AnnotatorRef> = HashMap::new();

        let mut test_string = "Hello $unknown:world".to_string();
        let result = annotate_string(&mut test_string, &annotators).unwrap();

        assert!(!result);
        assert_eq!(test_string, "Hello $unknown:world");
    }

    #[test]
    fn test_annotate_string_mixed_placeholders() {
        let mut annotators: HashMap<String, AnnotatorRef> = HashMap::new();
        let annotator = Rc::new(RefCell::new(
            Box::new(CapitalizeAnnotator) as Box<dyn Annotater>
        ));
        annotators.insert("cap".to_string(), annotator);

        let mut test_string = "Process $cap:main with $other:value and $cap:thread".to_string();
        let result = annotate_string(&mut test_string, &annotators).unwrap();

        assert!(result);
        assert_eq!(test_string, "Process MAIN with $other:value and THREAD");
    }
}
