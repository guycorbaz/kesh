# Story 15.7b : La démonstration et la remise à zéro laissent leur trace

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni critères, ni tâches, ni tests.** Elle ne garde que
les pointeurs vers ses deux volets et l'historique des passes qui ont conduit au découpage. *(La
définition du statut `split` l'impose ; précédents : 15-7, 15-7a, 15-1, 15-5.)* La version complète
d'avant découpage se lit au commit `9bbc52de`.

## Les deux sous-stories (et la 15-7b3, née de la seconde)

| | fiche | ce qu'elle porte | issues |
|---|---|---|---|
| **15-7b1** | `15-7b1-trace-demonstration.md` | **Volet A** — `seed-demo` : `installation.demo_seeded` puis l'étape dans la dernière transaction du peuplement, acteur threadé, une action et ses libellés, registre 103 → 104, manuel | `refs #434` |
| **15-7b2** | `15-7b2-remise-a-zero.md` | **Volet B** — `reset` : une transaction rejouée sur interblocage, trois gardes sous son verrou, connexion fermée à la libération, `installation.reset` en dernier ; identité de la société préservée et règle unique des principaux orphelins (clés révoquées puis repointées, C-15-7-45), y compris sans société (#528) ; vidage dérivé de la liste canonique, partition gardée sur le schéma par trois règles (#279) ; une action, registre 104 → 105, manuel ; bootstrap gardé (cause de #542) | `closes #434`, `closes #279` ; `refs #528`, `refs #542`, `refs #540`, `refs #538` |
| **15-7b3** | `15-7b3-reparation-des-installations-atteintes.md` | **Née de la 15-7b2** à sa P3 (C-15-7-40) — réparation des installations déjà atteintes par #528 (au démarrage et à la restauration) et par #542 (au démarrage) ; clés orphelines révoquées puis repointées (C-15-7-45, C-15-7-46) | `closes #528`, `closes #542` |

Ordre : 15-7a1 → 15-7a2 → 15-7b1 → 15-7b2, chacune après le merge de la précédente ; **les trois
dernières dans la même release** (C-15-7-25, C-15-7-31). La 15-7b3 suit la 15-7b2 ; même release visée, sans contrainte de sûreté (C-15-7-44).

## Pourquoi le découpage

La section « Dérogation règle de splitting » de cette fiche (écrite à la P3, C-15-7-28) prévoyait :
« si une passe suivante remonte un défaut **né d'une remédiation** à sévérité égale, la coupe Volet A /
Volet B s'applique sans nouvel arbitrage ». La passe P4 a établi les deux conditions : **R4-1 = F-1**
(MEDIUM) est né de la remédiation P3 (`9bbc52de`, ligne ajoutée au test 5) ; et **R4-2** a montré que
la dérogation reposait elle-même sur un constat faux — trois des quatre MEDIUM de la P3 venaient des
correctifs de la P2 (vérifié par versions de la fiche : `retry_with` et `KEY_COLUMN_USAGE` absents au
commit `3846b206`, présents au commit `3c82e58f` ; « ou à leur défaut » retiré par la P2). C'était donc
un **recyclage** au sens de l'amendement D5. L'orchestrateur a appliqué la coupe (choix **C-15-7-31**).
Les remédiations de la P4 ont été appliquées **dans les deux fiches filles**, pas ici ; leur tableau est
au Change Log de la 15-7b2.

## Change Log

- 2026-10-08 — Née du découpage de la 15-7 à la passe de validation P1 (choix C-15-7-8). Reprend
  les AC 4 et 5 de la 15-7, remédiés : F-1 (les trois gardes dans la transaction qui efface, C-15-7-11),
  R2/F-4 (compteurs de `demo_seeded` par les variantes `_in_tx`, boucle de retry autour de la dernière
  transaction, C-15-7-14), R11 (`is_stub` dans la transaction), R1/F-2 (tests 2 et 8 par déclencheur),
  R4/F-7 (tests 3, 6, 7, 9), F-8 (400 et non 409), F-10 (lecture verrouillante, connexion détachée),
  R10 (rollback explicite, hypothèse d'auto-incrément), F-11 (`tables_cleared` exact), F-13
  (`serde_json` dans `kesh-seed`). **Élargie par décision de l'orchestrateur** : ferme #528
  (identité de la société préservée, C-15-7-9) et, par conséquence nécessaire, #279 (vidage dérivé de
  la liste canonique, C-15-7-10). Recompte d'alors : 12 AC, 9 tâches, 11 tests.
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-7b-validate-prompt-p2.md` ; deux
  lentilles en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b-p2-{R,F}.md`). Bruts, recomptés depuis les rapports : R **3 MEDIUM, 10 LOW** ;
  F **5 MEDIUM, 8 LOW**. Après fusion des doublons (R-2 = F-3, R-3 = F-4, R-8 absorbé par F-1, L-2
  absorbé par R-6, la colonne `country` de L-7 absorbée par R-13) : **0 CRITICAL, 0 HIGH, 6 MEDIUM,
  16 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R-1 | MEDIUM | R | le grep de l'AC 10 ne voit ni `seed_demo` ni `is_stub` ; commentaire de la route `reset` perdu au découpage | AC 10 : grep élargi, sites attribués (index) |
  | R-2 = F-3 | MEDIUM | R, F | l'ordre des `DELETE` contredit le Pattern 5 ; interblocage possible ; Dev Note muette | AC 2 (ordre écrit), AC 10 (liste d'exceptions), Dev Note, `retry_with` — C-15-7-22 (amende C-15-7-12) |
  | R-3 = F-4 | MEDIUM | R, F | test 8 ne prouve pas la santé de la connexion | test 8 : pool à une connexion, `@@SESSION.foreign_key_checks`, mutation ; `detach()` angle mort assumé |
  | F-1 (+ R-8) | MEDIUM | F, R | #528 sans société : 500 à la remise à zéro, branche « aucune société » qui recrée sans rattacher | helper unique `attach_all_principals_in_tx`, `insert_stub` partagé, stub inséré sur base vide — C-15-7-23 ; tests 6b, 6c, 6d |
  | F-2 | MEDIUM | F | le vidage dérivé classe d'office toute table future dans « à effacer » | test 10b sur le schéma, liste fermée d'exceptions — C-15-7-24 |
  | F-5 | MEDIUM | F | le seul E2E de la remise à zéro est un échec attendu ; `isStub` change sans test | T9 (KF-029, cause signalée), AC 4 (`isStub: true`), tests 5 et 12 |
  | R-4 | LOW | R | « neuf `DELETE` » : huit | inventaire § 3, mutation du test 4 |
  | R-5 | LOW | R | lignes décalées (`:276`, `bootstrap.rs:35-36`, `backup.rs:33-86`, `accounts.rs:1020`) | corrigées |
  | R-6 (+ L-2) | LOW | R, F | précédent mal cité ; cas « commit puis échec du `SET` » indécis | AC 6 : contrôles rétablis avant le commit, cas supprimé — C-15-7-25 |
  | R-7 | LOW | R | ce que gardent les deux handlers n'est pas écrit | AC 1, AC 2, T4 |
  | R-9 | LOW | R | test 10 en partie tautologique | test 10 annoté ; 10b est la garde |
  | R-10 | LOW | R | emplacement de `RESET_*` laissé au choix | `kesh_db::backup` (AC 3) |
  | R-11 | LOW | R | déclencheur en protocole préparé, non vérifié | `sqlx::raw_sql` (ici et 15-7a2) |
  | R-12 | LOW | R | décomptes des transactions de `seed_demo` | inventaire § 2, AC 1, #43 |
  | R-13 (+ `country` de L-7) | LOW | R, F | `isStub` de la réponse ; colonne `country` | AC 4 |
  | L-1 | LOW | F | `delete_state` sans autre appelant de production | AC 2.6, AC 10 |
  | L-3 | LOW | F | grep du PDF raté sur l'apostrophe | AC 11 (`séquence d.installation`) |
  | L-4 | LOW | F | quatrième copie du `SELECT … onboarding_state FOR UPDATE` | `lock_state_in_tx` à la 15-7a1, employé ici |
  | L-5 | LOW | F | « la remise à zéro les efface » pas toujours vrai (#43) | Dev Notes |
  | L-6 | LOW | F | plus de cinq modules | décompte déclaré (huit), signal D5 |
  | L-7 | LOW | F | utilisateurs et clés de la démonstration conservés : « revoyez-les » | AC 11 |
  | L-8 | LOW | F | le dialogue de confirmation sous-estime l'effacement | signalé (CR), hors périmètre |

  **Trouvés pendant la remédiation** : (a) `restore_tables_in_tx` rétablit les contrôles **avant** le
  commit, ce qui supprime le cas que R-6 demandait de trancher ; (b) l'`INSERT` du stub existait en
  deux copies (`bootstrap.rs:347-360`, `routes/onboarding.rs:870-875`) — la troisième qu'aurait ajoutée
  `reset_demo` est évitée par `companies::insert_stub` ; (c) recompte sur le squash : 32 des 33 tables
  vidées atteignent `companies` par clés, `audit_log` seule exception. Propagation : grep des symptômes
  (`neuf DELETE`, `exactement une`, `:277`, `36-37`, `34-79`, `trois premières`) sur les trois fiches,
  l'index et le registre. Recompte : **12 AC, 9 tâches, 16 tests** (1 à 12, dont 6a-6d et 10b).
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-7b-validate-prompt-p3.md` ; deux
  lentilles Sonnet en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b-p3-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **3 MEDIUM, 6 LOW** ; F **4 MEDIUM, 4 LOW**. Après fusion (R-1 = F-2, R-2 = F-1, R-3 = F-3, R-4 = F-5,
  R-6 absorbe F-8 a/b) : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 9 LOW distincts**. Sévérité : P2 6 MEDIUM →
  P3 4 MEDIUM, **distincts** de ceux de la P2 et portant sur la conception d'origine (aucun né d'un
  correctif de la P2) — signal D5 non levé au sens de l'amendement de 2026-10-06, déclaré quand même.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R-1 = F-2 | MEDIUM | R, F | `reset_to_stub_in_tx` écrit `NULL` dans quatre colonnes `NOT NULL DEFAULT ''` | AC 4 : sept nullables à `NULL`, `address_*` à `''`, `address_country`/`country` à `'CH'` (squash vérifié) ; test 5 colonne à colonne — C-15-7-28 |
  | R-2 = F-1 | MEDIUM | R, F | le prédicat de rejeu ne voit pas les erreurs sqlx brutes | AC 2 : `ResetAttemptError` sans `From<sqlx::Error>` (tout par `map_db_error`, `begin`/`commit` compris), `is_reset_retryable` ; test 13 sur un vrai 1213 — C-15-7-27 |
  | R-3 = F-3 | MEDIUM | R, F | « atteint `companies` » par fermeture ordinaire : règle 2 vide, mutation sans effet | AC 3 : chaîne sans traverser une table préservée, `audit_log` exception nommée (aucune FK) ; recalculé sur le squash ; mutations — C-15-7-28 (amende C-15-7-24) |
  | F-4 | MEDIUM | F | `user-manual.tex:2216` et `admin-manual.tex:2244` hors du tableau ; grep sensible à la casse | AC 11 : deux lignes ajoutées, grep `-niE` |
  | R-4 = F-5 | LOW | R, F | jeton déjà émis porte l'id mort après réparation de #528 | AC 4 (limite), AC 11 (manuel), tests 6b/6c par jeton neuf |
  | R-5 | LOW | R | renvois de ligne d'avant la 15-7a2 ; commentaire `:866-869` co-cité | base `b74c3dac` déclarée, attribution à cette fiche |
  | R-6 (+ F-8 a, b) | LOW | R, F | `ResetOutcome`, `None` du verrou, langue d'`insert_stub`, T2/T3 | AC 1, AC 2, AC 4, T2, T3 |
  | R-7 | LOW | R | test 12 : état de module non injectable | test 12 : API mockée, `resetDemo()` avant `render` |
  | R-8 | LOW | R | raison d'être du rejeu de `seed_demo` changée | AC 10 : doc-comment « conservé par prudence » |
  | R-9 | LOW | R | test 11 vert dès avant la story | test 11 annoté |
  | F-6 | LOW | F | pas de section de dérogation au découpage | section « Dérogation règle de splitting » — C-15-7-28 |
  | F-7 | LOW | F | fenêtre #43 élargie par la préservation de la société | Dev Notes ; commentaire sur #43 signalé |
  | F-8 (c, d) | LOW | F | `DemoBanner` rend tout 403 en « Seul un administrateur » ; statut | signalé (même CR que L-8) ; convention de statut écrite |

  Propagation : grep des symptômes (`address_*`, `is_deadlock_error`, `atteint`, `deux familles`,
  `594-598`, `ResetOutcome`) sur les trois fiches, l'index et le registre. Recompte : **12 AC,
  9 tâches, 17 tests** (1 à 13, dont 6a-6d et 10b).
- 2026-10-08 — **Passe de validation P4** (prompt versionné `15-7b-validate-prompt-p4.md` ; deux
  lentilles Opus en contexte frais ; rapports `target/gate-logs/15-7b-p4-{R,F}.md`, non versionnés) :
  **0 CRITICAL, 0 HIGH, 6 MEDIUM, 11 LOW distincts** (R 3 MEDIUM, 6 LOW ; F 4 MEDIUM, 7 LOW, bruts
  recomptés). ⚠️ **Rectification du Change Log de la P3** (R4-2) : « aucun né d'un correctif de la P2 »
  est **faux** — R-2 = F-1 (`retry_with`, introduit par C-15-7-22), R-3 = F-3 (test 10b, introduit par
  C-15-7-24) et R-1 = F-2 (« ou à leur défaut » supprimé par la remédiation de R-13) sont nés des
  correctifs de la P2 ; seul F-4 relevait de la conception d'origine. Le signal D5 de la P3 était donc
  un recyclage. **Signal D5 constaté à la P4, et suivi** : **découpée** en 15-7b1 (Volet A) + 15-7b2
  (Volet B), C-15-7-31. Corps vidé ; remédiations appliquées dans les fiches filles. Recompte d'avant
  découpage : 12 AC, 9 tâches, 17 tests ; après : 15-7b1 **7 AC, 7 tâches, 4 tests** ; 15-7b2 **11 AC,
  8 tâches, 13 tests** (AC 7 à 12 partagés entre les deux volets, numérotation conservée).
