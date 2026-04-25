// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use chardetng::EncodingDetector;
use flate2::{read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::common::OutputMode;

#[derive(Serialize, Deserialize)]
pub struct CommandResultData {
    pub timestamp: String,
    pub command: String,
    pub status: bool,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
}

impl CommandResultData {
    pub fn generate_result(&self, is_compress: bool) -> CommandResult {
        let output = self.output.as_bytes().to_vec();
        let stdout = self.stdout.as_bytes().to_vec();
        let stderr = self.stderr.as_bytes().to_vec();

        CommandResult {
            timestamp: self.timestamp.clone(),
            command: self.command.clone(),
            status: self.status,
            is_compress,
            output: vec![],
            stdout: vec![],
            stderr: vec![],
        }
        .set_output(output)
        .set_stdout(stdout)
        .set_stderr(stderr)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CommandResult {
    pub timestamp: String,
    pub command: String,
    pub status: bool,
    pub is_compress: bool,
    pub output: Vec<u8>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            timestamp: String::default(),
            command: String::default(),
            status: true,
            is_compress: false,
            output: vec![],
            stdout: vec![],
            stderr: vec![],
        }
    }
}

impl PartialEq for CommandResult {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.status == other.status
            && self.output == other.output
            && self.stdout == other.stdout
            && self.stderr == other.stderr
    }
}

impl CommandResult {
    fn set_data(&self, data: Vec<u8>, data_type: OutputMode) -> Self {
        let u8_data = if self.is_compress {
            let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&data).unwrap();
            encoder.finish().unwrap()
        } else {
            data
        };

        match data_type {
            OutputMode::Output => CommandResult {
                output: u8_data,
                ..self.clone()
            },
            OutputMode::Stdout => CommandResult {
                stdout: u8_data,
                ..self.clone()
            },
            OutputMode::Stderr => CommandResult {
                stderr: u8_data,
                ..self.clone()
            },
        }
    }

    pub fn set_output(&self, data: Vec<u8>) -> Self {
        self.set_data(data, OutputMode::Output)
    }

    pub fn set_stdout(&self, data: Vec<u8>) -> Self {
        self.set_data(data, OutputMode::Stdout)
    }

    pub fn set_stderr(&self, data: Vec<u8>) -> Self {
        self.set_data(data, OutputMode::Stderr)
    }

    fn get_data(&self, data_type: OutputMode) -> String {
        let data = match data_type {
            OutputMode::Output => &self.output,
            OutputMode::Stdout => &self.stdout,
            OutputMode::Stderr => &self.stderr,
        };

        if self.is_compress {
            let mut decoder = GzDecoder::new(&data[..]);
            let mut decoded = Vec::new();
            let _ = decoder.read_to_end(&mut decoded);
            decode_bytes(&decoded)
        } else {
            decode_bytes(data)
        }
    }

    pub fn get_output(&self) -> String {
        self.get_data(OutputMode::Output)
    }

    pub fn get_stdout(&self) -> String {
        self.get_data(OutputMode::Stdout)
    }

    pub fn get_stderr(&self) -> String {
        self.get_data(OutputMode::Stderr)
    }

    pub fn export_data(&self) -> CommandResultData {
        CommandResultData {
            timestamp: self.timestamp.clone(),
            command: self.command.clone(),
            status: self.status,
            output: self.get_output(),
            stdout: self.get_stdout(),
            stderr: self.get_stderr(),
        }
    }
}

pub(super) fn decode_bytes(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut detector = EncodingDetector::new();
    detector.feed(data, true);
    let encoding = detector.guess(None, true);
    let (cow, _, _) = encoding.decode(data);
    cow.into_owned()
}
