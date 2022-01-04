mod blockchain;
use blockchain::Blockchain;

fn main() {
    let mut blnch: Blockchain = Blockchain::new(); //override ctor?

    blnch.new_transaction(3);

    blnch.mint();
    blnch.mint();

    println!("{:?}", blnch.save("serialized_blockchain".to_string()));
    println!(
        "{:#?}",
        Blockchain::load("serialized_blockchain".to_string()).expect("Cant load file!")
    );

    println!("\nBlockchain len: {}", blnch.chain.len());
}
