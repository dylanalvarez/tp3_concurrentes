use crate::blockchain::{Blockchain, BlockchainRecord};
use crate::blockchain_node::BlockchainNode;

#[derive(PartialEq, Debug)]
pub enum BlockchainMessage {
    AskForBlockchain,
    BlockchainResult(Blockchain),
}

impl BlockchainMessage {
    pub fn as_string(self) -> String {
        match self {
            BlockchainMessage::AskForBlockchain => {
                format!("AskForBlockchain")
            }
            BlockchainMessage::BlockchainResult(blockchain) => {
                format!("BlockchainResult:{}", blockchain.as_str())
            }
        }
    }

    /// Example: BlockchainResult:asd,10.0,1234;qwe,9.0,5125
    pub fn from_string(string: String) -> Option<BlockchainMessage> {
        let tokens = string.split(":").collect::<Vec<&str>>();
        match tokens[0] {
            "AskForBlockchain" => Some(BlockchainMessage::AskForBlockchain),
            "BlockchainResult" => Some(BlockchainMessage::BlockchainResult(Blockchain::from_str(
                tokens[1].to_string(),
            ))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ask_for_blockchain_as_string() {
        assert_eq!(
            *b"AskForBlockchain",
            BlockchainMessage::AskForBlockchain.as_string().as_bytes()
        );
    }

    #[test]
    fn test_blockchain_result_as_string() {
        let mut blockchain = Blockchain::new();
        let record = BlockchainRecord {
            student_name: "asd".to_string(),
            grade: 10.5,
            hash: 123,
        };
        blockchain.add_record(record);
        assert_eq!(
            String::from("BlockchainResult:asd,10.5,123"),
            BlockchainMessage::BlockchainResult(blockchain).as_string()
        );
    }

    #[test]
    fn test_ask_for_blockchain_from_string() {
        assert_eq!(
            BlockchainMessage::from_string(String::from("AskForBlockchain")),
            Some(BlockchainMessage::AskForBlockchain)
        );
    }

    #[test]
    fn test_blockchain_result_from_string() {
        let mut expected = Blockchain::new();
        let record = BlockchainRecord {
            student_name: "asd".to_string(),
            grade: 10.5,
            hash: 1234,
        };
        expected.add_record(record);
        assert_eq!(
            BlockchainMessage::from_string(String::from("BlockchainResult:asd,10.5,1234")),
            Some(BlockchainMessage::BlockchainResult(expected))
        );
    }

    #[test]
    fn test_none_from_string() {
        assert_eq!(
            BlockchainMessage::from_string(String::from("asdadasd")),
            None
        );
    }

    #[test]
    fn test_empty_blockchain_result_as_string() {
        let mut blockchain = Blockchain::new();
        assert_eq!(
            String::from("BlockchainResult:"),
            BlockchainMessage::BlockchainResult(blockchain).as_string()
        );
    }

    #[test]
    fn test_empty_blockchain_result_from_string() {
        let mut expected = Blockchain::new();
        assert_eq!(
            BlockchainMessage::from_string(String::from("BlockchainResult:")),
            Some(BlockchainMessage::BlockchainResult(expected))
        );
    }

    #[test]
    fn test_blockchain_result_multiple_records_as_string() {
        let mut blockchain = Blockchain::new();
        let a_record = BlockchainRecord {
            student_name: "asd".to_string(),
            grade: 10.5,
            hash: 123,
        };
        let another_record = BlockchainRecord {
            student_name: "qwe".to_string(),
            grade: 8.5,
            hash: 678,
        };
        blockchain.add_record(a_record);
        blockchain.add_record(another_record);
        assert_eq!(
            String::from("BlockchainResult:asd,10.5,123;qwe,8.5,678"),
            BlockchainMessage::BlockchainResult(blockchain).as_string()
        );
    }

    #[test]
    fn test_blockchain_result_multiple_records_from_string() {
        let mut expected = Blockchain::new();
        let record = BlockchainRecord {
            student_name: "asd".to_string(),
            grade: 10.5,
            hash: 123,
        };
        let another_record = BlockchainRecord {
            student_name: "qwe".to_string(),
            grade: 8.5,
            hash: 678,
        };
        expected.add_record(record);
        expected.add_record(another_record);
        assert_eq!(
            BlockchainMessage::from_string(String::from(
                "BlockchainResult:asd,10.5,123;qwe,8.5,678"
            )),
            Some(BlockchainMessage::BlockchainResult(expected))
        );
    }
}
