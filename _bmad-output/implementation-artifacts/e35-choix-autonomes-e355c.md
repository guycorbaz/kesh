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

## C-e355c-5 — `ajouterJoursIso` refuse ce qui n'est pas `AAAA-MM-JJ`

- **Contexte** : D3 fixe le comportement sur des jours valides ; rien sur une entrée mal formée.
- **Retenu** : une exception nommant le format attendu (`/^(\d{4})-(\d{2})-(\d{2})$/`), testée ;
  une spec qui passerait un instant (`…T12:00:00Z`) ou un jour mal formé échoue fort au lieu de
  produire `NaN-NaN-NaN`.
- **Autres options** : accepter et rendre une valeur indéfinie (échec muet plus loin).
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
- **Autres options** : lire l'heure réelle (`jour_suisse(Utc::now())`) pour cette seule
  précondition — vert sous tout instant, mais une seconde horloge dans un test, ce que D1 exclut ;
  rendre la borne et la date de facture relatives au jour (change les fixtures d'un test de la
  24-4c, et relève de #1125, tests à dates fixes de 2026).
- **Réversibilité** : totale.

## C-e355c-7 — C-e355b-5 (`services_rollout::ac11_a…`) hors du périmètre de la fiche

- **Contexte** : le prompt demande de traiter C-e355b-5 s'il entre dans le périmètre. La fiche ne
  nomme pas `services_rollout.rs` ; la 35-5c ne le touche pas ; AC5 ne lance les deux instants de
  la fiche que sur les binaires que la story touche, et la passe « nuit réelle »
  (`$(date -u +%F)T23:30:00Z`, postérieure au 1er juin 2026) le laisse vert.
- **Retenu** : non traité ici ; dit au rapport (déjà commenté sur #1125).
- **Autres options** : passer un `asOf` explicite au montage (retire au test sa lecture à l'`asOf`
  par défaut, C-e355b-5).
- **Réversibilité** : sans objet.
