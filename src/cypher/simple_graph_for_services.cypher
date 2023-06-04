create constraint simple_graph_for_services if not exists 
for () -[s: serves]-> () 
require (s.service, s.topic) is unique