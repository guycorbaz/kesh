# Epic 35 — registre des choix autonomes de la Story 35-5c (clé `e355c`)

Registre de l'agent de développement (Opus 5.5, worktree `kesh-e355c`, voie 2), tenu par union
(`docs/agents-orchestration.md` § 1). Chaque choix : contexte, option retenue, autres options,
réversibilité. Option prudente et réversible par défaut.

## C-e355c-1 — Le jour de `setup()` (`journal_entries.rs`) : `jour_suisse(Utc::now())`, non `HorlogeFixe(Utc::now()).aujourd_hui()`

- **Contexte** : D1 écrit, pour un `mod tests` de `src`, `HorlogeFixe(Utc::now()).aujourd_hui()`
  en ligne. Or la garde de durcissement de la 35-5a0 (`date_du_jour_garde.rs`, `HorlogeFixe`,
  portée `Src`) n'admet `HorlogeFixe` que dans `kesh-core/src/horloge.rs`,
  `kesh-api/src/horloge.rs` et `kesh-api/src/middleware/auth.rs` : l'écrire dans
  `journal_entries.rs` la ferait rougir. La fiche elle-même cite l'autre forme au LOW V1-8 de sa
  validation (« `jour_suisse(Utc::now())` en ligne dans les `mod tests` de `src` »).
- **Retenu** : `let today = kesh_core::horloge::jour_suisse(chrono::Utc::now());` dans `setup()`,
  seule lecture de l'horloge du module. Même valeur que `HorlogeFixe(Utc::now()).aujourd_hui()`
  (`Horloge::aujourd_hui` rend `jour_suisse(self.maintenant())`, `kesh-core/src/horloge.rs:31-33`).
  Le contrôle de M6 s'adapte : le motif `Utc::now\(\)\s*\.(date_naive|naive_utc)` ne rend plus
  rien sur le module, et le motif nu `Utc::now` n'y rend que `setup()` et le suffixe d'unicité
  `timestamp_nanos_opt` (Change Log).
- **Autres options** : ajouter `journal_entries.rs` à la liste `HorlogeFixe` de la garde (élargit
  la plomberie admise en `src` pour un module de tests) ; garder la forme de D1 et rougir.
- **Réversibilité** : totale (tests seuls).

## C-e355c-2 — Les trois helpers à `let jour` et `lettrer_dans` prennent le jour de `setup()`

- **Contexte** : `supprimer_sous_borne`, `supprimer_avec`, `la_devalidation_voit_l_exercice_posterieur`
  relisaient l'horloge dans une variable `jour` ; `lettrer_dans` aussi, sans recevoir le jour.
- **Retenu** : les trois lient `jour` directement depuis `setup()` (`let (…, jour) = setup(&pool)`) ;
  `lettrer_dans` reçoit un paramètre `jour: NaiveDate`, que ses trois appelants passent depuis
  leur `setup()` ; la borne du verrou de `delete_in_tx_of_a_lettered_entry_in_a_locked_period_says_period_locked`
  se lie au même jour, et `test_create_rejects_closed_fiscal_year` recrée l'exercice de
  `today.year()`.
- **Autres options** : un second `let jour = today;` (bruit) ; laisser `lettrer_dans` relire
  l'horloge (rouvre la course de minuit que D1 ferme).
- **Réversibilité** : totale.

## C-e355c-3 — La garde des specs balaie tous les `.ts` de `tests/e2e`, tests des helpers compris

- **Contexte** : D4 dit « `tests/e2e/**` » sans préciser si les `*.test.ts` des helpers
  (`contrast.test.ts`, `test-state.test.ts`, le nouveau `dates.test.ts`) en font partie.
- **Retenu** : tous les `.ts`, sans exclusion — l'option la plus stricte ; aucun ne porte de motif
  aujourd'hui. Les motifs s'appliquent au texte entier (forme coupée vue) et le relevé prend le
  numéro de la ligne où la forme commence ; une ligne ne donne qu'un relevé. La garde lit aussi
  les commentaires, comme sa moitié `src` : `dates.ts` ne cite donc aucun motif dans sa
  documentation.
- **Autres options** : exclure les `*.test.ts` (comme la moitié `src`), au risque d'un test de
  helper qui prendrait l'horloge Node sans être vu.
- **Réversibilité** : totale.

## C-e355c-4 — Les fonctions locales `todayZurich()` de deux specs délèguent à `todayInZurich()`

- **Contexte** : `bank-manual-statement.spec.ts:65` et `open-items.spec.ts:38` définissent chacune
  un `todayZurich()` qui lisait l'horloge (forme juste, mais seconde lecture de l'horloge des specs).
- **Retenu** : le corps devient `return todayInZurich();` ; les appels restent tels quels (diff
  minimal, hunks distincts de ceux d'autres stories sur ces fichiers).
- **Autres options** : retirer la fonction locale et renommer chaque appel (diff plus large, sans
  gain de preuve : la garde ne voit plus aucune lecture de l'horloge hors `todayInZurich`).
- **Réversibilité** : totale.
- *(Annoté le 2026-11-14, remédiation de la revue P1, B6 : l'autre option est retenue finalement —
  les alias d'une ligne étaient du bruit sans valeur propre. `todayZurich()` est retiré de
  `bank-manual-statement.spec.ts` et d'`open-items.spec.ts`, et `today()` des trois
  `invoices-settle*.spec.ts` (non nommés ci-dessus, même forme) ; chaque appel devient
  `todayInZurich()`. Commit `7fcede7e8`.)*

## C-e355c-5 — `ajouterJoursIso` refuse ce qui n'est pas `AAAA-MM-JJ`

- **Contexte** : D3 fixe le comportement sur des jours valides ; rien sur une entrée mal formée.
- **Retenu** : une exception nommant le format attendu (`/^(\d{4})-(\d{2})-(\d{2})$/`), testée ;
  une spec qui passerait un instant (`…T12:00:00Z`) ou un jour mal formé échoue fort au lieu de
  produire `NaN-NaN-NaN`.
- **Autres options** : accepter et rendre une valeur indéfinie (échec muet plus loin).
- *(Annoté le 2026-11-14, remédiation de la revue P1, A5/B4/E3 : la promesse « échoue fort au lieu
  de `NaN-NaN-NaN` » ne valait que pour une entrée mal formée — un `n` `NaN` rendait encore
  `NaN-NaN-NaN`, une année `0050` devenait 1950 (`Date.UTC`), `2026-02-30` se normalisait en
  silence. `ajouterJoursIso` refuse désormais aussi une année hors de 1000 à 9999, à l'entrée comme
  au résultat ; un jour qui ne se relit pas à l'identique (contrôle aller-retour) ; un `n` qui
  n'est pas un entier (`Number.isInteger` : `NaN`, `0.5`, `Infinity`). Un test par cas
  (`dates.test.ts`, 6 → 9 tests), rouges sur le `dates.ts` de `40090cdd3`
  (`kesh-gate-logs/e355c/p1-rouge-dates.log`). Commit `7fcede7e8`.)*
- **Réversibilité** : totale.

## C-e355c-6 — La précondition « borne passée » d'`admin_full_import_e2e` lit l'horloge de test

- **Contexte** : `validating_a_backdated_invoice_is_refused_by_the_period_lock` pose une borne fixe
  `2026-06-30` et asserte qu'elle est passée (« il pourrira sinon en 2026 »). Convertie à
  `aujourd_hui_test()` (D1), elle rougit sous un `KESH_TEST_INSTANT` antérieur au 30 juin 2026 —
  la classe (b) de la 35-5a (instant antérieur à une date fixe du montage), non un mélange de
  jours ; à l'heure réelle, postérieure, elle est verte et ne recule pas.
- **Retenu** : `aujourd_hui_test()`, comme les deux autres sites du fichier : la précondition
  porte sur le « aujourd'hui » des tests, celui du serveur. Le rouge sous les deux instants de la
  fiche est nommé au journal de nuit et au Change Log, avec les autres rouges de classe (b).
- **Constaté à l'exécution** (`kesh-gate-logs/e355c/nuit.log`, commit `40090cdd3`) : sous B1 et B2,
  le binaire rend **deux** rouges de cette classe — celui-ci (`:2282`) et
  `full_import_letters_a_reopened_historical_piece` (`:2009`, borne `2026-03-31`), converti par la
  35-5a0 (1a9af93b9) ; les deux restent rouges **de jour** au 11 mars 2026 (B3) et sont verts au
  15 juillet 2026 (J1) et sous la nuit réelle (N1). **Écart à la lettre d'AC5** (« verts sous les
  deux instants »), déclaré ; à proposer en commentaire de #1125 par l'orchestrateur.
- **Autres options** : lire l'heure réelle (`jour_suisse(Utc::now())`) pour cette seule
  précondition — vert sous tout instant, mais une seconde horloge dans un test, ce que D1 exclut ;
  rendre la borne et la date de facture relatives au jour (change les fixtures d'un test de la
  24-4c, et relève de #1125, tests à dates fixes de 2026).
- **Réversibilité** : totale.
- *(Annoté le 2026-11-14, remédiation de la revue P1, A3/E7 : la déclaration est gardée, option
  prudente — rendre la borne relative changerait l'objet du test (une facture antidatée sous une
  borne de verrou **réelle** de 2026), ce qui relève de #1125. Précision :
  `validating_a_backdated_invoice_is_refused_by_the_period_lock` doit ce rouge à la conversion
  faite par **cette** story (sur `ebd5b654c`, sa précondition lisait l'heure réelle et restait
  verte sous tout instant) ; `full_import_letters_a_reopened_historical_piece` le doit à la 35-5a0.
  AC5 était inatteignable à la lettre sans #1125, les instants de la fiche (2025-12-31, 2026-03-10)
  précédant les bornes fixes de 2026. Les deux tests sont commentés sur #1125 (fait par
  l'orchestrateur le 2026-11-14). L'alternative « lire l'heure réelle » n'a pas tourné : « vert sous
  tout instant » s'entend d'après la lecture de la ligne, non d'une exécution.)*

## C-e355c-7 — C-e355b-5 (`services_rollout::ac11_a…`) hors du périmètre de la fiche

- **Contexte** : le prompt demande de traiter C-e355b-5 s'il entre dans le périmètre. La fiche ne
  nomme pas `services_rollout.rs` ; la 35-5c ne le touche pas ; AC5 ne lance les deux instants de
  la fiche que sur les binaires que la story touche, et la passe « nuit réelle »
  (`$(date -u +%F)T23:30:00Z`, postérieure au 1er juin 2026) le laisse vert.
- **Retenu** : non traité ici ; dit au rapport (déjà commenté sur #1125).
- **Autres options** : passer un `asOf` explicite au montage (retire au test sa lecture à l'`asOf`
  par défaut, C-e355b-5).
- **Réversibilité** : sans objet.

## C-e355c-8 — La garde « horloge réelle d'un test » : liste close de **deux** entrées, portée `src` et `tests`

- **Contexte** : revue P1, B2 (MEDIUM) et E6 (LOW) : `jour_suisse(Utc::now())` (C-e355c-1) et
  `HorlogeFixe(Utc::now())` dans `tests/` échappent aux deux familles de la garde Rust ; un test
  de `kesh-api` qui copierait `setup()` serait sourd à `KESH_TEST_INSTANT` sans rien faire rougir.
  L'orchestrateur demandait une liste close d'**une** entrée (`journal_entries.rs`). Le détecteur,
  joué d'abord à liste vide sur l'arbre (`kesh-gate-logs/e355c/p1-rouge-garde-rust.log`), en trouve
  **deux** : `kesh-db/src/repositories/journal_entries.rs:3246` et
  `kesh-api/src/middleware/auth.rs:302`.
- **Retenu** : une garde de durcissement de plus dans `date_du_jour_garde.rs` (`HORLOGE_REELLE`,
  `HORLOGE_REELLE_ADMISES`, test `horloge_reelle_d_un_test`), comptée sur le texte entier sans ses
  lignes `//` (`\s` couvre les retours à la ligne : forme coupée vue), sur **tout** `src` et
  `tests` — en production, la forme contournerait aussi l'horloge de l'état, et `src` n'en porte
  aucune. Liste close à égalité stricte : `journal_entries.rs` 1 (ligne lue :
  `let today = kesh_core::horloge::jour_suisse(chrono::Utc::now());`, le binaire `lib` de `kesh-db`
  ne lit pas la variable) et `middleware/auth.rs` 1 (ligne lue :
  `horloge: std::sync::Arc::new(kesh_core::horloge::HorlogeFixe(chrono::Utc::now())),`, `test_state()`
  du `mod tests`, choix de la 35-5a0 commenté aux lignes 300-301 ; les tests du middleware ne datent
  rien au jour). Sondes : qualifiée et coupée dans `tests/`, `HorlogeFixe(Utc::now())` dans
  `tests/`, une seconde lecture dans `journal_entries.rs` ; vert pour un instant donné, un
  `Utc::now().timestamp()`, un commentaire. Angle mort écrit dans l'en-tête de la garde : variable
  intermédiaire, alias, enveloppe, `SystemTime`, `impl Horloge` ad hoc.
- **Autres options** : n'admettre que `journal_entries.rs` et convertir `auth.rs:302` à
  `crate::horloge::horloge_de_test()` — touche un `mod tests` de production et la liste de la garde
  `horloge_de_test` (portée `src`, deux entrées), hors du périmètre d'une remédiation ; restreindre
  la portée aux seuls `tests/` (la suggestion de B2) — laisserait les `mod tests` de `src` sans
  contrôle, où se trouve justement le site d'origine ; écrire la forme à l'angle mort seulement.
- **Réversibilité** : totale (un test).

## C-e355c-9 — La garde des specs : motifs étendus, appariement à la ligne entière

- **Contexte** : revue P1, E4 (une seconde lecture ajoutée à une ligne admise passait sous son
  entrée, l'appariement se faisant par `includes`) et E5/B3 (`new Date;`, `new globalThis.Date()`,
  `Date()` sans `new`, `.format()` sans argument échappaient aux motifs).
- **Retenu** : `auditer(…, exact)` — pour les specs, l'entrée égale la ligne relevée sans son
  indentation (ce que D4 disait déjà) ; la moitié `src` garde l'appariement par extrait. Quatre
  motifs de plus : `new (globalThis.)Date` non suivi de `(`, `new globalThis.Date()`,
  `Date()` sans `new` (précédé d'aucun caractère de mot ni `$` : `getDate()`, `toDate()` exclus,
  `globalThis.Date()` vu), `.format()` sans argument. Aucun site actuel n'est relevé de plus
  (`git grep` sur `tests/e2e`, l'audit des specs reste à trois entrées). Sondes ajoutées, rouges
  sans l'extension (`p1-rouge-garde-front.log`). Restent écrits à l'angle mort : instant passé par
  une variable, `new Date(...[])`, `new Date(+Date.now())`, alias de `Date`, `Reflect.construct`, et
  le jour relu plusieurs fois dans une spec (B5, non fermé : risque inhérent, le navigateur et le
  serveur lisant aussi leur horloge).
- **Autres options** : écrire les formes à l'angle mort sans les relever ; un `const jour` par test
  dans chaque spec (B5) — diff large sur des specs que l'E2E de la clôture seul exercerait.
- **Réversibilité** : totale.
