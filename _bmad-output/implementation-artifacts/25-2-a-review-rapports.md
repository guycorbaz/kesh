# Story 25-2-a — rapports des passes de revue

⚠️ **Pourquoi ce fichier existe.** La **passe 2 l'a exigé** : aucun artefact de la passe 1 n'était
versionné, si bien que sa déclaration d'axes — pourtant faite — n'était vérifiable par personne.
La règle du dépôt veut qu'*une passe qui ne déclare pas ses axes exercés ne compte pas comme
passe* ; l'esprit en est que la déclaration soit **contestable**, pas seulement qu'elle ait eu
lieu. Une déclaration qui vit dans une notification de session disparaît avec elle.

*Manquement de l'orchestrateur, relevé par la passe qu'il avait lui-même mandatée.*

---

## Passe 1 — Sonnet, contexte frais, périmètre `main...HEAD`

**Verdict : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW.**

### Axes déclarés exercés

1. **Correction de la garde** — corps complet d'`accounts::update` lu **dans le fichier**, pas
   seulement dans le diff ; ordre confirmé (no-op → version → rôle singleton → garde → `UPDATE`).
   Dépôt grepé pour d'autres écritures d'`account_type` : **un seul site, un seul appelant**.
   Chemin de restauration de sauvegarde examiné et écarté comme légitime.
2. **Exactitude du compte** — `journal_entries.fiscal_year_id` vérifié `NOT NULL` et jamais réécrit,
   ce qui rend exacte la somme par exercice. Filtres `company_id` et statut contrôlés.
3. **Audit** — les deux branches tracées ; aucun changement de type n'échappe au journal.
4. **Manuel** — `.tex` **et PDF aplati** (`pdftotext | tr`), phrase par phrase ; manuel admin grepé,
   aucune mention obsolète.
5. **Propagation** — `account_type` / `accountType` sur tout le dépôt ; recherche spécifique d'un
   commentaire reprenant l'argument TOCTOU de #274 dans `opening_balances.rs` : **aucun n'existe**.
6. **Frontend** — `readRetypeImpact` et `+page.svelte` lus en entier ; les 5 tests Vitest **rejoués
   réellement**.
7. **Décomptes** — **recomptés depuis la source** : 24 → 37 tests dépôt, 14 → 19 API, 5 au frontend
   = **+23** ; ventilation i18n recomptée par diff des sites, **7 nets** dont `accounts-updated` à 0.
8. **Dette #439** — `git cat-file` sur `main` **et** `origin/main`, `gh pr view 439` : dette réelle,
   conséquence bénigne (code brut affiché, pas d'erreur).

Transverses non mandatés mais exécutés : `cargo fmt --check`, `clippy -D warnings`, `cargo check`,
`npm run check`. Non exécutés, et déclarés tels : suites complètes et E2E (interdites par le mandat).

### Findings et suites données

| Sév. | Finding | Suite |
|---|---|---|
| MEDIUM | `Status: ready-for-dev` contredisait ses tâches, toutes cochées ; convention du dépôt : `review` | **corrigé** |
| LOW | `readRetypeImpact` ne validait qu'`entryCount`, dégradant les trois autres champs en silence — commentaire plus affirmatif que le code | **corrigé** : les quatre invalident |
| LOW | Aucun scénario Playwright du parcours navigateur | **assumé sur mesure** — aucun preset de seed ne pose d'écriture (`test_endpoints.rs:19`) ; inventorié en Dev Notes |
| LOW | Fenêtre TOCTOU de `POST /opening-balances` non fermée | **inventorié** — résidu pré-existant, ni créé ni aggravé ici |
| LOW | Deux fiches de stories sœurs dans le diff | observation ; à mentionner dans la PR |

---

## Passe 2 — Haiku, **ciblée**, périmètre `d32a0244` seul

Prompt versionné : `25-2-a-review-prompt-p2-ciblee.md`. Diff **unique et aplati**, conformément au
garde-fou du dépôt sur l'indexation multi-commits.

**Verdict : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.**

### Axes déclarés exercés — les quatre, aucun laissé de côté

1. **Régression** — contrat serveur (`kesh-db/src/errors.rs`) confronté à la validation client :
   correspondance exacte des quatre types. Aucun appelant n'envoie de `details` partiel, le serveur
   en étant l'unique source.
2. **Tests** — les cinq de `accounts-page.test.ts` : quatre envoient les quatre champs, le cinquième
   invalide **pour la raison qu'il annonce**. Pas de vacuité.
3. **Déclarations** — vérifiées dans la limite du commit ; **réserve émise** sur l'absence d'artefact
   de la passe 1 (à l'origine de ce fichier).
4. **Propagation et commentaires** — `readRetypeImpact` n'a **aucun autre appelant** ; le commentaire
   décrit exactement le comportement, ni plus ni moins.

### Contrôle de l'orchestrateur — un « 0 finding » se vérifie comme un finding

Reprise indépendante du point le plus risqué de son verdict, la **sérialisation** JSON et non la
seule déclaration Rust : les quatre champs sont émis **inconditionnellement** par le `json!` de
`crates/kesh-api/src/errors.rs`, aucun n'est `Option`, aucun ne porte de `skip_serializing`. Le
durcissement de la validation ne peut donc pas rejeter un refus légitime.

---

## Clôture

La remédiation de la passe 2 **ne touche aucune ligne de code de production** — elle se réduit à ce
fichier. C'est le critère de clôture de la boucle, et il est satisfait.
