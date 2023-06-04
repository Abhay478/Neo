# Stuff
- The `main.rs` file and `auth_nt` directory are pretty much good to go. Entire stack is built.
  - We could add new `Authorities` should the need arise.
  - Possible parameter tweaking (cookie expiry).
- The `actix_nt.rs` file is pretty thin. This is because most of the business logic has moved inside the `impl`s for `Query` and `Mutation`.
  - Contains a `graphql` endpoint, accepting `Queries`, `Mutations` and `Subscriptions`. This route handles all incoming traffic not related to authentication - those are in `auth_nt/handlers.rs`
  - This is mostly from the `async-graphql` and `async-graphql-actix-web` crates.
  - Still a little iffy on the mechanics.
- Added a `graphql_nt.rs` file, with structs for `Queries`, `Mutations` and `Subscriptions`.
  - These structs will have functions that call the functions in `neo_nt.rs`.
  - Authorization verification occurs here (only).
  - This is the actual server.
- `neo_nt` is the neo4j part, off to a good start. All structs in `models.rs`, all functions in `handlers.rs` (looking a little bloated, might change).
  - Can add several more functionalities.

# Schema so far
- ```(: Account) -[: follows]-> (: Topic) <-[: serves]- (: Service) ```
- ```(: Topic) -[: contains]-> (: Page)```
- ```(: Account) -[: provides]-> (: Service)```