use std::collections::HashMap;
use std::{fmt, fs, error};

#[derive(Debug)]
pub struct BuildError {
    msg: String,
}

#[derive(Debug)]
pub struct RunError {
    msg: String,
}

impl BuildError {
    fn new(msg: String) -> BuildError {
        BuildError { msg: msg }
    }
}

impl RunError {
    fn new(msg: String) -> RunError {
        RunError { msg: msg }
    }
}

impl error::Error for BuildError {}
impl error::Error for RunError {}


impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Build error: {}", self.msg)
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Run error: {}", self.msg)
    }
}

pub type Program = Vec<BfToken>;

pub struct BFI;

#[derive(Clone, Debug)]
pub enum BfToken {
    IncermentDataPtr,
    DecrementDataPtr,
    IncrementCurrent,
    DecrementCurrent,
    OutputByte,
    InputByte,
    JumpStart(usize),
    JumpEnd(usize),
}

impl BFI {
    pub fn build(code: String) -> Result<Program, BuildError> {
        println!("Building...");
        // Tokenize
        let bf_commands = "<>+-.,[]";
        let code: Vec<char> = code
        .chars()
        .filter(|&c| bf_commands.contains(c))
        .collect();
        //println!("Code: {:?}", &code);
        let mut program: Program = Vec::new();
        let mut jump_locations: HashMap<usize, usize> = HashMap::new();
        for (i, c) in code.iter().enumerate() {
            let token = match c {
                '>' => BfToken::IncermentDataPtr,
                '<' => BfToken::DecrementDataPtr,
                '+' => BfToken::IncrementCurrent,
                '-' => BfToken::DecrementCurrent,
                '.' => BfToken::OutputByte,
                ',' => BfToken::InputByte,
                '[' => {
                    let jump_end_location = Self::locate_jump_end(i, &code[i+1..])?;
                    jump_locations.insert(jump_end_location, i);
                    BfToken::JumpStart(jump_end_location+1)
                },
                ']' => {
                    if let Some(&jump_start_location) = jump_locations.get(&i) {
                        BfToken::JumpEnd(jump_start_location+1)
                    } else {
                        return Err(BuildError::new(format!("Failed to find jump start for {i}")));
                    }
                },
                _ => { return Err(BuildError::new(format!("Unexpected token at {i}"))); },
            };
            program.push(token);
        }
        println!("{:?}", jump_locations);
        println!("Successfully built");
        Ok(program)
    }

    pub fn run(program: Program) -> Result<(), RunError> {
        println!("Running...");
        let mut memory: [i32; 30000] = [0; 30000];
        let mut instruction_ptr: usize = 0;
        let mut data_ptr: usize = 0;

        loop {
            if instruction_ptr >= program.len() {
                break;
            }
            match program[instruction_ptr] {
                BfToken::IncermentDataPtr => data_ptr += 1,
                BfToken::DecrementDataPtr => data_ptr -= 1,
                BfToken::IncrementCurrent => memory[data_ptr] += 1,
                BfToken::DecrementCurrent => memory[data_ptr] -= 1,
                BfToken::OutputByte => print!("{}", memory[data_ptr] as u8 as char),
                BfToken::InputByte => {
                    let mut buff = String::new();
                    if let Err(err) = std::io::stdin().read_line(&mut buff) {
                        return Err(RunError::new(format!("User input failure at {} with {}", instruction_ptr, err)));
                    }
                    memory[data_ptr] = buff.trim().parse::<i32>().unwrap();
                },
                BfToken::JumpStart(jump_to) => {
                    if memory[data_ptr] == 0 {
                        instruction_ptr = jump_to;
                        continue;
                    }
                },
                BfToken::JumpEnd(jump_to) => {
                    if memory[data_ptr] != 0 {
                        instruction_ptr = jump_to;
                        continue;
                    }
                },
            }
            instruction_ptr += 1;
        }
        println!();
        Ok(())
    }

    fn locate_jump_end(jump_start_location: usize, program: &[char]) -> Result<usize, BuildError> {
        let mut to_skip: usize = 0;
        let mut curr_inst: usize = jump_start_location + 1;
        for c in program {
            match c {
                '[' => {
                    to_skip += 1;
                }
                ']' if to_skip > 0 => {
                    to_skip -= 1;
                },
                ']' => {
                    return Ok(curr_inst);
                }
                _ => (),
            };
            curr_inst += 1;
        }
        Err(BuildError::new(format!("Failed to find end of jump at instruction {jump_start_location}")))
    }
}

pub fn read_file_from_args(mut args: impl Iterator<Item = String>) -> Result<String, Box<dyn error::Error>> {
    args.next();
    let filename = args.next().ok_or_else(|| "Filepath is not supplied in args")?;
    Ok(fs::read_to_string(filename)?)
}