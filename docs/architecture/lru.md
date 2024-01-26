# LRU Strategy

Right now our db engine is very simple, it stores the data in a map and that's it, but we have LRU strategy.
With this strategy we can set a limit of item to keep in memory and when the limit is reached the oldest item is deleted.
This is implemented with a linked list where the head is the oldest item and the tail is the newest item. 
Every time a key is accessed it is moved to the tail of the list. 
When the limit is reached the head will be deleted and the new item will be added to the tail.
