create constraint unique_topics if not exists 
for (t: Topic) 
require t.name is unique
