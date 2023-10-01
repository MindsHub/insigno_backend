## Comandi utili

Eseguire server:
```sh
cargo run
```

Eseguire server che si aggiorna in automatico quando si cambiano file:
```sh
cargo watch -x run
```

Cose da installare per fare test:
```sh
cargo install cargo-watch
cargo install cargo-tarpaulin
cargo install diesel_cli --no-default-features --features "postgres"
```

Per aggiungere una migrazione:
```sh
diesel migration generate MIGRATION_NAME
```

### PostgreSQL

Aprire shell di `psql`:
```sh
psql -h insigno.mindshub.it -U mindshub insignorocketdb
```

Numero connessioni attive al momento:
```sql
SELECT COUNT(*) FROM pg_stat_activity;
```

Killare tutte le connessioni che provengono dalla stessa macchina da cui si è connessi con la shell (eccetto la shell da cui si sta eseguendo il comando)
```sql
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND client_addr IN (SELECT client_addr FROM pg_stat_activity WHERE pid = pg_backend_pid());
```
Per aprire la documentazione è necessario eseguire lo script document nella home. In particolare genera una nuova documentazione con il comando corretto e la apra sul browser predefinito. Questo perché siamo in super beta e la documentazione cambia giornalmente
