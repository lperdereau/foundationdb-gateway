# Start db

```bash
docker run -d --name myfdb -p 4500:4500 foundationdb/foundationdb:7.3.59
docker exec myfdb bash -c ""/usr/bin/fdbcli -C /var/fdb/fdb.cluster --exec 'configure new single memory'""
```


# Run Redis

```bash
echo "docker:docker@${container_ip}:4500"
cargo run --bin redisgw
```
