create constraint simple_graph_for_subs if not exists 
for () -[f: follows]-> () 
require (f.sub, f.topic) is unique