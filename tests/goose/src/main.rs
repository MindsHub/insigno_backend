use goose::prelude::*;

async fn loadtest_index(user: &mut GooseUser) -> TransactionResult {
    let params = [("email", "test@test.test"), ("name", "ilTester"), ("password", "Testtest1)")];
    let resp = user.post_form("signup", &params).await?;
    let resp = resp.response.unwrap();
    assert_eq!(resp.status().as_u16(), 200u16);
    let text = resp.text().await.unwrap();
    assert_eq!(&text, "mail inviata");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
    .set_default(GooseDefault::Host, "http://localhost:8000").unwrap()
        .register_scenario(scenario!("LoadtestTransactions")
            .register_transaction(transaction!(loadtest_index))
        )
        
        .execute()
        .await?
        ;

    Ok(())
}
