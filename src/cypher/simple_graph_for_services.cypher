create constraint simple_graph_for_services if not exists 
for () -[s: serves]-> () 
require (s.serv, s.topic) is unique