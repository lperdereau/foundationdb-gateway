# Plan de tests pour redisgw

Ce fichier énumère les cas de tests proposés pour la passerelle Redis (`redisgw`). Il sert de source de vérité pour prioriser et implémenter les tests unitaires et end-to-end.

Format :
- `Nom du test` : description courte — remarques d'implémentation

## Organisation

Les tests sont classés en deux catégories :

- Unit tests : tests rapides, isolés, exécutés via `cargo test` et ciblant la logique métier (data model, opérations atomiques, parsing RESP, TTL passive, etc.). Ces tests doivent être rapides et ne pas dépendre d'un worker externe.
- End-to-end (e2e) tests : tests d'intégration qui démarrent un conteneur FoundationDB (via `fdb_testcontainer`) et lancent le serveur `redisgw`. Ils valident le comportement sur TCP RESP et des interactions plus larges (persistance, TTL active, multi-key). Ces tests sont plus lents et isolés.

## Priorité haute — Unit tests

- test_set_get_simple : Vérifie que `SET` puis `GET` renvoie la valeur attendue.
- test_set_overwrite : `SET key v1`, `SET key v2` puis `GET == v2`.
- test_set_nx_xx : Vérifier `NX` (ne set pas si existe) et `XX` (ne set pas si absent).
- test_del_multiple : `DEL` sur plusieurs clés retourne le bon compte.
- test_getdel_atomic : `GETDEL` retourne la valeur et supprime la clé.
- test_incr_decr_basic : `INCR`/`DECR` enchaînés, valeurs négatives, etc.
- test_concurrent_incr : lancer N tâches `INCR` concurrentes sur la même clé et vérifier atomicité (résultat == initial+N). (Unit + concurrency)

## Strings - cas complémentaires (Unit)

- test_append_empty_and_nonempty : `APPEND` sur clé absente crée la clé; sur existante concatène et retourne la longueur.
- test_mget_mset : `MSET` suivi de `MGET` retourne l'ensemble des valeurs.
- test_set_get_with_getflag : `SET ... GET` retourne l'ancienne valeur quand `GET` présent.
- test_empty_key_or_value : comportement pour clé vide / valeur vide.
- test_max_value_size : limite supérieure (100KB) et chunking via `DataModel`.

## TTL / expiration - cas détaillés (Unit)

- test_ttl_negative_edge : TTL déjà passé (EXAT dans le passé) doit rendre la clé inexistante immédiatement.
- test_ttl_deleted_on_get : si TTL expiré, `GET` supprime paresseusement la clé.
- test_set_with_get_flag_and_ttl : combiner `SET GET` et `EX` pour vérifier ancienne valeur retournée.

## End-to-end (E2E) tests — priorité haute

- test_set_with_ttl_ex_px : tester `EX`, `PX`, `EXAT`, `PXAT` (durées en s et ms, timestamp absolu) — vérifier expiration passive et active (e2e recommandé pour active-cleaner).
- test_ttl_active_cleaner_behavior : insérer plusieurs clés avec TTL et s'assurer qu'un worker nettoie celles-ci (e2e).
- test_multi_exec_atomicity : `MULTI`/`EXEC` applique atomiquement une suite d'opérations (e2e).
- test_persistence_across_reopen : écriture, recréation du client `FoundationDB` et lecture (vérifier durabilité) (e2e).

## Hashes, Lists, Sets, Sorted Sets (Unit + E2E selon complexité)

- Hashes (HSET/HGET/HGETALL/HDEL/HINCRBY) — implémentation initiale en Unit, vérifications multi-field en E2E.
- Lists (LPUSH/RPUSH/LPOP/RPOP/LLEN/LRANGE) — Unit for behavior, E2E for ordering under concurrency.
- Sets (SADD/SMEMBERS/SREM/SISMEMBER) — Unit for correctness, E2E for large sets and persistence.
- Sorted Sets (ZADD/ZRANGE/ZREM/ZSCORE) — Unit for semantics, E2E for range queries at scale.

## Concurrency / stress (E2E preferred)

- test_concurrent_set_and_get : concurrence mixte `SET`/`GET` sur mêmes clés.
- test_concurrent_set_with_ttl : course entre `SET ... EX` et `GET`.

## Error / boundary cases (Unit)

- test_invalid_incr_on_nonint : `INCR` sur valeur non-int retourne erreur.
- test_long_key_name : gestion de clés très longues.
- test_invalid_args_count : appeler une commande avec mauvais nombre d'arguments retourne erreur RESP appropriée.

---

Priorité recommandée pour implémentation :
1. Unit : Strings fondamentaux (set/get, nx/xx, append) + INCR/DECR
2. Unit : TTL passive et erreurs (INCR non-int)
3. E2E : TTL active (cleaner), persistance, multi-key atomicity
4. Unit/E2E : Hashes, Lists, Sets (basiques)
5. E2E : Sorted sets et opérations multi-key avancées

Notes d'implémentation
- Tests unitaires doivent utiliser `fdb_testcontainer::get_db_once()` pour un environnement FDB éphémère.
- Préférer des clés uniques par test pour éviter interférences (p.ex. inclure le nom du test dans la clé).
- Éviter tests lents (longs sleeps); privilégier TTL courts (ms) quand possible. Marquer explicitement les tests e2e plus lents.

Comment exécuter

Exécuter tous les tests du crate `redisgw` (unit + e2e) :

```bash
cd /path/to/foundationdb-gateway
cargo test -p redisgw -- --nocapture
```

Exécuter un test spécifique (par nom) :

```bash
cargo test -p redisgw test_concurrent_incr -- --nocapture
```

Notes
- Les tests e2e requièrent que Docker soit disponible pour `fdb_testcontainer`.
- Si tu veux, j'implémente automatiquement les N premiers tests prioritaires (indiquer 3, 5 ou 10) et je les soumets en patch.
