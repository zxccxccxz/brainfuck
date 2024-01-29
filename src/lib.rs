use std::collections::HashMap;
use std::fs;

#[derive(Clone)]
enum BfToken {
    IncrDataPtr,
    DecrDataPtr,
    IncrCurrent,
    DecrCurrent,
    OutputByte,
    InputByte,
    JumpStart(usize),
    JumpEnd(usize),
    Unrecognized,
}

pub struct BFI {
    program: Vec<BfToken>,
    memory: [i32; 30000],
    inst_ptr: usize,
    data_ptr: usize,
    jump_locations: HashMap<usize, usize>, // end -> start
}

impl BfToken {
    fn is_valid(&self) -> bool {
        !matches!(*self, BfToken::Unrecognized)
    }
}

impl BFI {
    pub fn new() -> BFI {
        BFI {
            program: Vec::new(),
            memory: [0; 30000],
            inst_ptr: 0,
            data_ptr: 0,
            jump_locations: HashMap::new(),
        }
    }

    fn read_file_from_args(mut args: impl Iterator<Item = String>) -> Result<String, String> {
        args.next();
        let filename = match args.next() {
            Some(filename) => filename,
            None => {
                return Err(format!("Not enough arguments, please provide file which contains brainfuck code."));
            }
        };
        fs::read_to_string(filename).or_else(|err| Err(format!("Failed to locate file: {err}")))
    }

    fn locate_jump_end(jump_start_location: usize, program: &[BfToken]) -> Result<usize, String> {
        let mut to_skip: usize = 0;
        let mut curr_inst: usize = jump_start_location + 1;
        for c in program {
            match c {
                BfToken::JumpStart(_) => {
                    to_skip += 1;
                }
                BfToken::JumpEnd(_) if to_skip > 0 => {
                    to_skip -= 1;
                },
                BfToken::JumpEnd(_) => {
                    return Ok(curr_inst);
                }
                _ => (),
            };
            curr_inst += 1;
        }
        Err(format!("Failed to find end of jump at instruction {jump_start_location}"))
    }

    pub fn build(&mut self, args: impl Iterator<Item = String>) -> Result<(), String> {
        println!("Building...");
        let code = Self::read_file_from_args(args)?;
        // Tokenize
        self.program.clear();
        self.jump_locations.clear();
        let mut program = Vec::new();
        for c in code.bytes() {
            let token = match c as char {
                '>' => BfToken::IncrDataPtr,
                '<' => BfToken::DecrDataPtr,
                '+' => BfToken::IncrCurrent,
                '-' => BfToken::DecrCurrent,
                '.' => BfToken::OutputByte,
                ',' => BfToken::InputByte,
                '[' => BfToken::JumpStart(0),
                ']' => BfToken::JumpEnd(0),
                _ => BfToken::Unrecognized,
            };
            if token.is_valid() {
                program.push(token);
            }
        }
        // Handle jumps
        let mut curr_inst: usize = 0;
        self.program = program.clone();
        for inst in &mut self.program {
            match inst {
                BfToken::JumpStart(jump_to) => {
                    let end_location = Self::locate_jump_end(curr_inst, &program[curr_inst+1..])?;
                    self.jump_locations.insert(end_location, curr_inst);
                    *jump_to = end_location + 1;
                },
                BfToken::JumpEnd(jump_to) => {
                    if let Some(start_location) = self.jump_locations.get(&curr_inst) {
                        *jump_to = start_location + 1;
                    } else {
                        return Err(format!("Failed to find jump start at instruction {curr_inst}"));
                    }
                },
                _ => (),
            }
            curr_inst += 1;
        }
        println!("Successfully built");
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            if self.inst_ptr >= self.program.len() {
                break;
            }
            match self.program[self.inst_ptr] {
                BfToken::IncrDataPtr => self.data_ptr += 1,
                BfToken::DecrDataPtr => self.data_ptr -= 1,
                BfToken::IncrCurrent => self.memory[self.data_ptr] += 1,
                BfToken::DecrCurrent => self.memory[self.data_ptr] -= 1,
                BfToken::OutputByte => print!("{}", self.memory[self.data_ptr] as u8 as char),
                BfToken::InputByte => {
                    let mut buff = String::new();
                    if let Err(err) = std::io::stdin().read_line(&mut buff) {
                        return Err(format!("User input failure at {} with {}", self.inst_ptr, err));
                    }
                    self.memory[self.data_ptr] = buff.trim().parse::<i32>().unwrap();
                },
                BfToken::JumpStart(jump_to) => {
                    if self.memory[self.data_ptr] == 0 {
                        self.inst_ptr = jump_to;
                        continue;
                    }
                },
                BfToken::JumpEnd(jump_to) => {
                    if self.memory[self.data_ptr] != 0 {
                        self.inst_ptr = jump_to;
                        continue;
                    }
                },
                BfToken::Unrecognized => {
                    return Err(format!("Unexpected token at instruction {}", self.inst_ptr));
                }
            }
            self.inst_ptr += 1;
        }
        println!();
        Ok(())
    }
}