use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Debug)]
pub struct BlockchainRecord {
    pub student_name: String,
    pub grade: f64,
    pub hash: u64,
}

#[derive(PartialEq, Debug)]
pub struct Blockchain {
    records: Vec<BlockchainRecord>,
}

fn hash(a_string: String) -> u64 {
    let mut hasher = DefaultHasher::new();
    a_string.hash(&mut hasher);
    hasher.finish()
}

fn generate_hash(student_name: &String, grade: f64, previous_record_hash: u64) -> u64 {
    let mut to_be_hashed = student_name.clone();
    to_be_hashed.push_str(&grade.to_string());
    to_be_hashed.push_str(&previous_record_hash.to_string());
    hash(to_be_hashed)
}

fn is_valid(record: &BlockchainRecord, previous_record_hash: u64) -> bool {
    generate_hash(&record.student_name, record.grade, previous_record_hash) == record.hash
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        for record in &self.records {
            result.push_str(
                format!("{} {} {}\n", record.student_name, record.grade, record.hash).as_str(),
            )
        }
        write!(f, "{}", result)
    }
}

impl Blockchain {
    pub(crate) fn new() -> Blockchain {
        Blockchain {
            records: Vec::new(),
        }
    }
    pub fn clone(&self) -> Blockchain {
        Blockchain {
            records: self.records.clone(),
        }
    }

    pub fn last_record(&self) -> Option<&BlockchainRecord> {
        self.records.last()
    }

    pub fn add_grade(&mut self, student_name: String, grade: f64) {
        let previous_record_hash = match self.records.last() {
            None => 0,
            Some(record) => record.hash.clone(),
        };
        let hash = generate_hash(&student_name, grade, previous_record_hash);
        self.records.push(BlockchainRecord {
            student_name,
            grade,
            hash,
        });
    }

    pub fn add_record(&mut self, record: BlockchainRecord) {
        self.records.push(record)
    }

    pub fn is_valid(&self) -> bool {
        let mut last_hash = 0;
        for record in &self.records {
            if !is_valid(record, last_hash) {
                return false;
            }
            last_hash = generate_hash(&record.student_name, record.grade, last_hash);
        }
        true
    }

    pub fn from_str(blockchain_as_str: String) -> Blockchain {
        let splitted_blockhain = blockchain_as_str.split(";").into_iter();
        let mut new_blockchain = Blockchain::new();
        for record in splitted_blockhain {
            let fields = record.split(",").collect::<Vec<&str>>();
            if fields.len() > 2 {
                let record = BlockchainRecord {
                    student_name: fields[0].to_string(),
                    grade: fields[1].to_string().parse().unwrap(),
                    hash: fields[2].to_string().parse().unwrap(),
                };
                new_blockchain.add_record(record);
            }
        }
        new_blockchain
    }

    pub fn as_str(&self) -> String {
        let mut result = String::new();
        for record in &self.records {
            result.push_str(&*format!(
                "{},{},{};",
                record.student_name, record.grade, record.hash
            ));
        }
        result.pop();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_inputs_generate_same_outputs() {
        assert_eq!(
            generate_hash(&String::from("asdasd"), 6.5, 0),
            generate_hash(&String::from("asdasd"), 6.5, 0)
        )
    }

    #[test]
    fn test_empty_blockchain_is_valid() {
        assert_eq!(Blockchain::new().is_valid(), true)
    }

    #[test]
    fn test_add_grade_always_generates_valid_blockchains() {
        let mut blockchain = Blockchain::new();
        blockchain.add_grade(String::from("Dylan"), 10.0);
        blockchain.add_grade(String::from("Gustavo"), 7.99);
        assert_eq!(blockchain.is_valid(), true)
    }

    #[test]
    fn test_add_record_allows_for_invalid_blockchains() {
        let mut blockchain = Blockchain::new();
        blockchain.add_record(BlockchainRecord {
            student_name: String::from("Dylan"),
            grade: 10.0,
            hash: 0,
        });
        assert_eq!(blockchain.is_valid(), false)
    }

    #[test]
    fn test_add_record_needs_manual_hash_creation_for_valid_blockchains() {
        let mut blockchain = Blockchain::new();
        let student_name = String::from("Dylan");
        let grade = 10.0;
        blockchain.add_record(BlockchainRecord {
            student_name: student_name.clone(),
            grade,
            hash: generate_hash(&student_name, grade, 0),
        });
        assert_eq!(blockchain.is_valid(), true)
    }

    #[test]
    fn test_blockchain_hashes_are_recursive() {
        let mut blockchain = Blockchain::new();
        let first_student_name = String::from("Gustavo");
        let first_grade = 8.50;
        blockchain.add_grade(first_student_name.clone(), first_grade);
        let second_student_name = String::from("Dylan");
        let second_grade = 10.0;
        blockchain.add_record(BlockchainRecord {
            student_name: second_student_name.clone(),
            grade: second_grade,
            hash: generate_hash(
                &second_student_name,
                second_grade,
                generate_hash(&first_student_name, first_grade, 0),
            ),
        });
        assert_eq!(blockchain.is_valid(), true)
    }
}
