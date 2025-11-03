# Plan de tests pour redisgw

Ce fichier énumère les cas de tests proposés pour la passerelle Redis (`redisgw`). Il sert de source de vérité pour prioriser et implémenter les tests d'intégration/unitaires.

Format :
- `Nom du test` : description courte — remarques d'implémentation

## Priorité haute

- test_set_get_simple : Vérifie que `SET` puis `GET` renvoie la valeur attendue.
- test_set_overwrite : `SET key v1`, `SET key v2` puis `GET == v2`.
- test_set_nx_xx : Vérifier `NX` (ne set pas si existe) et `XX` (ne set pas si absent).
- test_del_multiple : `DEL` sur plusieurs clés retourne le bon compte.
- test_getdel_atomic : `GETDEL` retourne la valeur et supprime la clé.
- test_set_with_ttl_ex_px : tester `EX`, `PX`, `EXAT`, `PXAT` (durées en s et ms, timestamp absolu) — vérifier expiration passive et active.
- test_keep_ttl : `SET KEEPTTL` préserve le TTL existant.
- test_incr_decr_basic : `INCR`/`DECR` enchaînés, valeurs négatives, etc.
- test_concurrent_incr : lancer N tâches `INCR` concurrentes sur la même clé et vérifier atomicité (résultat == initial+N).

## Strings - cas complémentaires

- test_append_empty_and_nonempty : `APPEND` sur clé absente crée la clé; sur existante concatène et retourne la longueur.
- test_mget_mset : `MSET` suivi de `MGET` retourne l'ensemble des valeurs.
- test_set_get_with_getflag : `SET ... GET` retourne l'ancienne valeur quand `GET` présent.
- test_empty_key_or_value : comportement pour clé vide / valeur vide.
- test_max_value_size : limite supérieure (100KB) et chunking via `DataModel`.

## TTL / expiration - cas détaillés

- test_ttl_negative_edge : TTL déjà passé (EXAT dans le passé) doit rendre la clé inexistante immédiatement.
- test_ttl_deleted_on_get : si TTL expiré, `GET` supprime paresseusement la clé.
- test_ttl_active_cleaner_behavior : (si implémenté) insérer plusieurs clés avec TTL et s'assurer qu'un worker nettoie celles-ci.
- test_set_with_get_flag_and_ttl : combiner `SET GET` et `EX` pour vérifier ancienne valeur retournée.

## Hashes

- test_hset_hget_hgetall : `HSET` plusieurs champs puis `HGET`/`HGETALL` retournent correctement.
- test_hdel_and_hlen : `HDEL` retire champs; `HLEN` renvoie taille correcte.
- test_hincrby : `HINCRBY` sur champ entier; erreur si champ non-entier.

## Lists

- test_lpush_rpush_and_length : `LPUSH`/`RPUSH` et `LLEN`.
- test_lpop_rpop_order : vérifier ordre des `LPOP`/`RPOP`.
- test_lrange_indices_negative : `LRANGE` avec indices négatifs.
- test_lindex_out_of_range : `LINDEX` hors-bornes renvoie `Null`.

## Sets

- test_sadd_and_smembers : `SADD` puis `SMEMBERS` doit contenir tous les membres (ordre non garanti).
- test_srem_and_sismember : `SREM` et `SISMEMBER` reflètent la suppression.
- test_sunion_sinter_sdiff : opérations sur plusieurs ensembles.

## Sorted Sets

- test_zadd_zrange : `ZADD` puis `ZRANGE` retourne membres triés par score.
- test_zscore_and_update : `ZSCORE` et mise à jour de score.
- test_zrem : suppression de membres.

## Multi-key / transactions

- test_multi_exec_atomicity : `MULTI`/`EXEC` applique atomiquement une suite d'opérations.
- test_multi_discard : `MULTI` puis `DISCARD` ne modifie rien.
- test_watch_conflict : `WATCH` détecte une modification externe et `EXEC` échoue.

## Concurrency / stress

- test_concurrent_set_and_get : concurrence mixte `SET`/`GET` sur mêmes clés.
- test_concurrent_set_with_ttl : course entre `SET ... EX` et `GET`.

## Error / boundary cases

- test_invalid_incr_on_nonint : `INCR` sur valeur non-int retourne erreur.
- test_long_key_name : gestion de clés très longues.
- test_invalid_args_count : appeler une commande avec mauvais nombre d'arguments retourne erreur RESP appropriée.

## Persistence / reopen

- test_persistence_across_reopen : écriture, recréation du client `FoundationDB` et lecture (vérifier durabilité).

---

Priorité recommandée pour implémentation :
1. Strings fondamentaux (set/get, nx/xx, append)
2. TTL (EX/PX/EXAT/PXAT, KEEPTTL)
3. INCR/DECR + concurrence
4. Hashes, Lists, Sets (basiques)
5. Sorted sets et opérations multi-key

Notes d'implémentation
- Tests doivent utiliser `fdb_testcontainer::get_db_once()` comme dans les tests existants.
- Préférer des clés uniques par test pour éviter interférences.
- Éviter tests lents (longs sleeps); privilégier TTL courts (ms) quand possible.

Tu veux que j'implémente automatiquement les N premiers tests prioritaires dans `src/tests/` ? Si oui, combien (3, 5, 10) ?
