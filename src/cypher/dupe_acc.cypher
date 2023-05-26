match (acc:Account {username:$unm}) 
return count(acc) as count