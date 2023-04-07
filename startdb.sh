### From: https://github.com/toraritte/shell.nixes/blob/b4ef525b74703b04b3f90de9a6f6419f2a201043/elixir-phoenix-postgres/shell.nix

# Setup postgres data directory
if ! test -d $PGDATA
then
    pg_ctl initdb   \
    -o "--username=postgres"   \
    -D $PGDATA    \


    # In case of port already in use :
    # sed -i "s|^#port.*$|port = 5433|" $PGDATA/postgresql.conf
else
    EXIT
fi

# Setup hosts
HOST_COMMON="host\s\+all\s\+all"
sed -i "s|^$HOST_COMMON.*127.*$|host all all 0.0.0.0/0 trust|" $PGDATA/pg_hba.conf
sed -i "s|^$HOST_COMMON.*::1.*$|host all all ::/0 trust|"      $PGDATA/pg_hba.conf

echo "Starting postgres..."

# Start postgres
pg_ctl                                                    \
    -D $PGDATA                                            \
    -l $PGDATA/postgres.log                               \
    -o "-c unix_socket_directories='$PGDATA'"             \
    -o "-c listen_addresses='*'"                          \
    -o "-c log_destination='stderr'"                      \
    -o "-c logging_collector=on"                          \
    -o "-c log_directory='log'"                           \
    -o "-c log_filename='postgresql-%Y-%m-%d_%H%M%S.log'" \
    -o "-c log_min_messages=info"                         \
    -o "-c log_min_error_statement=info"                  \
    -o "-c log_connections=on"                            \
    start

# Setup database and dev user
psql -U postgres -h 127.0.0.1 -f setup.sql