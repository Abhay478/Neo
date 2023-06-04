# Stuff
- The `main.rs` and `auth_nt` are pretty much good to go. Entire stack is built.
  - We could add new `Authorities` should the need arise.
  - Possible parameter tweaking (cookie expiry).
- The `actix_nt.rs` file is basically empty. 
  - Will add a `graphql` endpoint, accepting `Queries`, `Mutations` and `Subscriptions`. 
  - This is mostly from the `async-graphql` and `async-graphql-actix-web` crates.
  - Still a little iffy on the mechanics.
- Added a `graphql_nt.rs` file, with structs for `Queries`, `Mutations` and `Subscriptions`.
  - These structs will have functions that call the functions in `neo_nt.rs`.
  - This is the actual server.
- `neo_nt.rs` is the neo4j part, off to a good start. 
  - Can add several more functionalities.

# Schema so far
- ```(: Account) -[: follows]-> (: Topic) <-[: serves]- (: Service) ```
- ```(: Topic) -[: contains]-> (: Page)```
- ```(: Account) -[: provides]-> (: Service)```