# Persistence

This is an optional feature that allows you to persist the state of the cache to disk.
The goal is to have a fast and reliable way to recover the state of the cache in case of a crash or a restart,
in this way you can avoid the need to rebuild the cache from scratch while your application is still responding to requests.

The design for this feature is to flush on disk based on how many changes we have in the cache, in this way we can avoid
useless writes to disk and we can keep the cache in memory as much as possible. 
You are free to tune the number of changes to trigger a flush on disk, this can impact the performance of the cache and
the IO of your system.

For now we just save as binary all entries for the hash table, and when we recover the state of the cache we just read
the file and load the entries in the hash table. This is a simple approach and it can be improved in the future.
The cons we have with this approach is that we need to rebuild the linked list used for LRU, then basically
we deserialize every entry and insert in a empty cache, the same path like application writing against the cache.
