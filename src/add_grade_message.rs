use crate::blockchain::BlockchainRecord;
use crate::blockchain_node::BlockchainNode;

#[derive(PartialEq, Debug)]
pub enum AddGradeMessage {
    FromCoordinator(BlockchainRecord),
    ToCoordinator(String, f64),
}

impl AddGradeMessage {
    pub fn as_string(self) -> String {
        match self {
            AddGradeMessage::FromCoordinator(record) => {
                format!("GRADE_FROM_COORDINATOR;{};{};{}", record.student_name, record.grade, record.hash)
            },
            AddGradeMessage::ToCoordinator(student_name, grade) => {
                format!("GRADE_TO_COORDINATOR;{};{}", student_name, grade)
            },
        }
    }

    pub fn from_string(string: String) -> Option<AddGradeMessage> {
        let tokens = string.split(";").collect::<Vec<&str>>();
        match tokens[0] {
            "GRADE_FROM_COORDINATOR" => {
                Some(
                    AddGradeMessage::FromCoordinator(
                        BlockchainRecord {
                            student_name: String::from(tokens[1]),
                            grade: tokens[2].parse::<f64>().unwrap(),
                            hash: tokens[3].parse::<u64>().unwrap(),
                        }
                    )
                )
            },
            "GRADE_TO_COORDINATOR" => {
                Some(
                    AddGradeMessage::ToCoordinator(
                        String::from(tokens[1]),
                        tokens[2].parse::<f64>().unwrap(),
                    )
                )
            },
            _ => {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_coordinator_as_string() {
        assert_eq!(*b"GRADE_FROM_COORDINATOR;asd;12.4;383838", AddGradeMessage::FromCoordinator(BlockchainRecord {
            student_name: "asd".to_string(),
            grade: 12.4,
            hash: 383838
        }).as_string().as_bytes());
    }

    #[test]
    fn test_to_coordinator_as_string() {
        assert_eq!(*b"GRADE_TO_COORDINATOR;qwe;52.6", AddGradeMessage::ToCoordinator(
            "qwe".to_string(),
            52.6
        ).as_string().as_bytes());
    }

    #[test]
    fn test_to_coordinator_from_string() {
        assert_eq!(
            AddGradeMessage::from_string(String::from("GRADE_TO_COORDINATOR;ueu;399.4")),
            Some(
                AddGradeMessage::ToCoordinator(
                    "ueu".to_string(),
                    399.4,
                )
            )
        );
    }

    #[test]
    fn test_from_coordinator_from_string() {
        assert_eq!(
            AddGradeMessage::from_string(String::from("GRADE_FROM_COORDINATOR;aaaa bbbb;123.123;9393939")),
            Some(
                AddGradeMessage::FromCoordinator(BlockchainRecord {
                    student_name: "aaaa bbbb".to_string(),
                    grade: 123.123,
                    hash: 9393939,
                })
            )
        );
    }

    #[test]
    fn test_none_from_string() {
        assert_eq!(
            AddGradeMessage::from_string(String::from("jfiosdjfiosdjio")),
            None
        );
    }
}
