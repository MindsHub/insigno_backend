## Comandi utili

### Rocket

Eseguire server:
```sh
cargo run
```

Eseguire server che si aggiorna in automatico quando si cambiano file:
```sh
cargo watch -x run
```

### PostgreSQL

Aprire shell di `psql`:
```sh
psql -h insignio.mindshub.it -U mindshub insigniorocketdb
```

Numero connessioni attive al momento:
```sql
SELECT COUNT(*) FROM pg_stat_activity;
```

Killare tutte le connessioni che provengono dalla stessa macchina da cui si è connessi con la shell (eccetto la shell da cui si sta eseguendo il comando)
```sql
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND client_addr IN (SELECT client_addr FROM pg_stat_activity WHERE pid = pg_backend_pid());
```
