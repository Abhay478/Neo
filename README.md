# Stuff
- The `main.rs` and `auth_nt` are pretty much good to go. Entire stack is built.
  - We could add new `Authorities` should the need arise.
  - Possible parameter tweaking
- The `actix_nt.rs` file is basically empty. 
  - Will add a `graphql` endpoint, accepting `Queries`, `Mutations` and `Subscriptions`.
- Will add a `juniper_nt.rs` file, with structs for `Queries`, `Mutations` and `Subscriptions`.
  - These structs will have functions that call the functions in `neo_nt.rs`.
- `neo_nt.rs` is the neo4j part, off to a good start. 
  - Can add several more functionalities.

# Schema so far
- ```(: Account) -[: follows]-> (: Topic) <-[: serves]- (: Service)```