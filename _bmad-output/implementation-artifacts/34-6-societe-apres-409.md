# Story 34-6 : après un 409, l'édition de la société montre l'écriture concurrente avant de l'écraser (#953)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration, aucun changement serveur.** Registre :
`e34-choix-autonomes-e343.md`, C-e343-6 et 7. Commits en `refs #953` ; à la PR : `closes #953`.*

## Story

En tant qu'Administrateur, je veux qu'une modification de la société faite entre-temps par quelqu'un d'autre me soit montrée
avant que mon enregistrement ne la remplace. Ainsi, je ne remplace pas sans le savoir l'e-mail, le téléphone, le site ou
l'adresse qu'un collègue vient de corriger, ni ma propre correction déjà enregistrée.

## Ce que le code dit (prémisses établies)

- **Les trois éditions de la société** sont dans `frontend/src/routes/(app)/settings/+page.svelte` :
  - e-mail : `startEmailEdit` et `saveEmail`, `:46-101` ;
  - téléphone et site : `startContactEdit` et `saveContactDetails`, `:113-163` ;
  - adresse : `startAddressEdit` et `saveAddress`, `:173-222`.

  Chacune envoie sa saisie avec `data.company.version`.
- **Le défaut est là.** Sur `OPTIMISTIC_LOCK_CONFLICT`, chacune relit la société (`data = await fetchCompanyCurrent()`,
  `:78`, `:143`, `:203`) et affiche « Conflit de version — les données ont été rechargées, réessayez ». Mais elle **garde la
  saisie** (`emailValue`, `phoneValue` et `websiteValue`, `addressForm`) et ne montre pas la valeur relue. Le clic suivant
  envoie la saisie avec la version fraîche : la route l'accepte et remplace l'écriture concurrente, que personne n'a vue.
- **Les tests affirment ce défaut.**
  - Vitest `routes/(app)/settings/settings-page.test.ts:289-310` : « le second essai part avec la version rechargée »
    (`version` 9) et réussit.
  - E2E `frontend/tests/e2e/company-address.spec.ts:128-157` : « le conflit de version est signalé, puis l'essai suivant
    passe ». La ville écrite par l'API, `Nyon`, y est écrasée sans être montrée.
- **Le serveur est juste.** Les trois routes reconstruisent le `CompanyUpdate` sur la version reçue et rendent 409 si elle est
  périmée (`crates/kesh-api/src/routes/companies.rs:112-170`, `:250-312`, `:344-380`). Chacune ne touche que ses champs
  (AC-7 de la 33-5, `tests/companies_e2e.rs:280`). Un no-op ne change pas la version (KF-004, `:164-165`, `:299-300`,
  `:373-374`).
- **Pourquoi la version peut changer sans que le bloc ait changé.** Une autre route de la société l'incrémente. Ce qui
  incrémente, relu dans `crates/kesh-db/src/repositories/companies.rs` :
  - les trois blocs (`update_in_tx`, `:909`) ;
  - la levée du drapeau provisoire (`clear_stub`, `:800`) ;
  - la remise à zéro (`:223`).

  **Le verrou de période ne l'incrémente pas** (`lock_books`, `:1043`, et `:1171` : `UPDATE companies SET
  books_locked_through = ? WHERE id = ?`). Le cas « aucun champ du bloc n'a changé » vient donc d'un autre bloc.
  *(Corrigé à la validation P1, 34-6-F2 : la fiche disait « probablement le verrou de période ».)*
- **Le cas de sa propre réponse perdue.** Une réponse qui n'arrive pas (erreur hors `ApiError` → `common-error`, `:93-97`)
  peut laisser la valeur écrite. Le second clic rend alors 409, et la valeur enregistrée égale la saisie. Aujourd'hui,
  l'écran le présente comme un conflit.
- **Les replis i18n** : `settings-company-{email,contact,address}-conflict` et `-conflict-reload-failed`
  (`crates/kesh-i18n/locales/fr-CH/messages.ftl:2350-2351,2540-2544`), aux quatre locales. La passe 6 de la 16-3a avait voulu
  des clés dédiées par bloc (`+page.svelte:137-141`).
- **Manuel** : *Vos coordonnées sur la facture* (`docs/manual/fr/user-manual.tex:1155`) et
  `docs/manual/fr/admin-manual.tex:2586`. Aucun ne décrit le conflit : **à vérifier au développement**, par un `grep` de
  « Conflit de version » et de « rechargées » dans les `.tex`.

## Conception (C-e343-6, C-e343-7)

- **La copie.** À l'ouverture de l'édition, chaque bloc garde une copie de ses valeurs enregistrées (`emailBase`,
  `contactBase`, `addressBase`).
- **Sur un 409**, on relit, puis on compare les champs **du bloc** :
  1. **Aucun n'a changé** depuis la copie : le message actuel. L'essai suivant est sans danger.
  2. **Tous égalent la saisie** : l'édition se ferme, `data` porte la société relue, et une notification dit « déjà
     enregistré ».
  3. **Sinon** : sous chaque champ en conflit, « Enregistré entre-temps : *valeur* », la valeur vide se disant « (vide) ». Un
     avis dit qu'enregistrer de nouveau remplacera ces valeurs. La saisie reste. La copie prend les valeurs relues, si bien
     qu'un second conflit se compare à elles.
- **Les définitions, champ par champ** (validation P1, 34-6-F1) :
  - `changed = { f : stored[f] ≠ base[f] ∧ stored[f] ≠ input[f] }`. Un champ modifié ailleurs **à la même valeur que la
    saisie** n'est pas en conflit : enregistrer ne le remplacera pas, et l'avis ne le nomme pas ;
  - `already-saved` si et seulement si `stored[f] = input[f]` pour tout champ du bloc ;
  - `unchanged` si `changed = ∅` et que le bloc n'est pas `already-saved` ;
  - `changed` sinon.
- **Une fonction pure** `companyEditConflict(base, stored, input, fields)` dans
  `frontend/src/lib/features/settings/company-conflict.ts`, qui rend `{ kind: 'unchanged' | 'already-saved' | 'changed',
  changed: string[] }`, et **un composant** `CompanyConflictNotice.svelte`. Les trois blocs les emploient.
- **Comparaison** sur les valeurs telles qu'envoyées : `trim()`, et chaîne vide ⇔ `null`, comme `saveEmail` et
  `saveContactDetails` (`:63-66`, `:126-130`). Pour l'adresse, comparaison sur les cinq composants. Le pays se compare
  **normalisé comme le serveur le range** (`normalized_country`, `crates/kesh-api/src/address_input.rs:37-43`) : `trim`, ASCII
  en majuscules, et vide ou absent valant `CH`. Un `stored.country = 'CH'` et une saisie vide sont donc égaux.
  `addressStructured` peut être `null` après relecture, si l'adresse a été retirée ailleurs (`?.` déjà employé à `:174`) : la
  copie et la comparaison le tolèrent, chaque composant valant alors vide.
- **Rien ne change côté serveur.**

## Acceptance Criteria

1. **AC-1 — le défaut de l'issue.** L'Administrateur ouvre l'édition de l'adresse. Un autre écrit, par l'API, la ville
   `Nyon`. L'Administrateur enregistre `Morges`. On attend le 409, puis la ville relue `Nyon` affichée sous le champ, la
   saisie `Morges` gardée et l'avis de remplacement. **Rien n'est écrit tant qu'il n'a pas cliqué de nouveau.** Second clic :
   `Morges` est écrit. **Rouge d'avant** : l'E2E réécrit à partir de `company-address.spec.ts:128-157`, où rien n'affiche
   `Nyon`.
2. **AC-2 — aucun champ du bloc n'a changé.** Un 409 dû à un autre bloc (le téléphone modifié pendant l'édition de l'adresse)
   donne le message actuel, sans valeur affichée. L'essai suivant aboutit.
3. **AC-3 — déjà enregistré.** Valeur relue égale à la saisie : l'édition se ferme sur « déjà enregistré ». Aucun second
   envoi.
4. **AC-4 — les trois blocs.** AC-1 à AC-3 valent pour l'e-mail (un champ), les coordonnées (deux champs, dont un seul peut
   avoir changé) et l'adresse (cinq composants).
5. **AC-5 — second conflit.** Après l'avis de l'AC-1, une troisième écriture concurrente : le second 409 montre la nouvelle
   valeur, comparée à la copie relue au premier conflit, et non à celle d'origine.
5 bis. **AC-5 bis — champ égal à la saisie** (34-6-F1). Bloc coordonnées : l'autre écrit le site `X`. L'Administrateur
   avait saisi le téléphone `T`, différent de la copie, et le même site `X`. Seul le téléphone est en conflit, s'il a changé
   ailleurs. Sinon le cas est `unchanged`. L'avis ne nomme jamais le site.
6. **AC-6 — relecture impossible.** Le message `-conflict-reload-failed` reste, et la saisie aussi. Aucun avis de valeur.
6 bis. **AC-6 bis — cycle de vie de l'avis** (34-6-F4). L'avis de conflit et les valeurs affichées sont remis à zéro par
   `start*Edit`, par l'annulation de l'édition et par un enregistrement réussi. Un second conflit remplace l'avis, sans
   l'empiler.
7. **AC-7 — textes.** Les clés neuves (`settings-company-conflict-changed` avec la variable `$value`,
   `settings-company-conflict-will-replace`, `settings-company-conflict-already-saved`, à nommer au développement) aux
   **quatre** locales, avec un repli Svelte identique. `lint-i18n-ownership` et les gardes de replis
   (`i18n-repli-divergent-actif.test.ts`, `i18n-keys.test.ts`) restent verts, et leurs compteurs sont recomptés. Les variables
   Fluent passent par le troisième argument de `msg` (`+page.svelte:23-34`), jamais par un `.replace()`.
8. **AC-8 — manuel.** Une phrase là où le manuel décrit l'édition de la société : un enregistrement refusé parce qu'un autre
   a modifié la société montre ce qui a été enregistré entre-temps, et un second clic remplace. Le PDF est régénéré et
   contrôlé aplati.

## Tâches

- [ ] **T1 — rouges d'abord**, journalisés dans `kesh-gate-logs/e343/rouges.log` :
  - Vitest de `company-conflict.ts` (rouge à l'import) ;
  - Vitest de `settings-page.test.ts` : le test `:289-310` récrit selon l'AC-1, et un test par bloc ;
  - E2E de l'AC-1.
- [ ] **T2** : `company-conflict.ts` et ses tests. On couvre les trois cas, puis un champ égal à la saisie (AC-5 bis), vide
  contre `null`, les espaces, un pays en minuscules, un pays vide contre `CH`, et `addressStructured` à `null`.
- [ ] **T3** : `CompanyConflictNotice.svelte`, les trois blocs, la copie prise à `start*Edit` et mise à jour au conflit.
- [ ] **T4** : i18n aux quatre locales, et les gardes de replis.
- [ ] **T5** : E2E. `company-address.spec.ts:128-157` est récrit, et un cas « coordonnées, un seul champ changé » s'ajoute
  à `company-contact-details.spec.ts`.
- [ ] **T6** : manuel et PDF ; captures de l'avis pour l'adresse et pour les coordonnées, en thème clair, dans
  `kesh-gate-logs/e343/captures/`.
- [ ] **T7** : CHANGELOG *Corrigé* (`refs #953`).

### Mutations nommées

- **M1** : `companyEditConflict` compare à la saisie au lieu de la copie → rouge du cas « aucun champ du bloc n'a changé ».
- **M2** : la copie n'est pas mise à jour au conflit → rouge AC-5.
- **M3** : le cas `already-saved` envoie quand même → rouge AC-3 (envoi compté par le mock).
- **M4** : la comparaison sans `trim` → rouge du test « espaces ».
- **M5** : le bloc des coordonnées ne compare que le téléphone → rouge AC-4 (site seul changé).

## Gates

Frontend entier (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; E2E complet au dernier commit de code. Aucun fichier
Rust n'est touché, hormis les `.ftl` : `cargo test -p kesh-i18n` et `textes_coherents` si une garde lit les clés.

## Ce qui ne se traite pas ici

- **Le verrou de période** (`submitLock`, `submitUnlock`, `:291` et `:306`) : il ne porte pas de saisie à écraser. Hors #953.
- **Les autres écrans à version** : même famille, non jugée ici. Inventaire : `git grep -n OPTIMISTIC_LOCK_CONFLICT --
  'frontend/src/routes' ':!*.test.ts'`, relevé à la validation P1. Il donne `settings/invoicing`, `settings/dunning`,
  `settings/api-keys`, `contacts`, `products`, `accounts`, `users` et `bank-accounts` (qui relit, 28-4b). `invoices/[id]` et
  `supplier-invoices/[id]` (et `/edit`) sont des fiches de pièce, hors de cette famille de formulaires. Le développeur lit
  chacun de ces écrans. Ce qui relit puis renvoie la saisie sans la montrer s'ouvre en issue, **une par écran** : la fiche
  ne le traite pas.

## Recoupements

- `settings/+page.svelte` : aucune autre story de l'Epic 34.
- `messages.ftl` : toutes les stories qui ajoutent des clés ; à fusionner par union.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : **validation P1 (Sonnet) : 1 M, 4 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p1-e343.md`. Chaque prémisse a été relue au code :
  - **F1 (M)** : définitions de `changed`, `already-saved` et `unchanged` champ par champ, et AC-5 bis.
  - **F2 (L)** : le verrou de période n'incrémente pas la version (`companies.rs:1043`, `:1171` lus). Prémisse et C-e343-6
    rectifiées.
  - **F3 (L)** : le pays normalisé comme le serveur (`address_input.rs:37-43` lu), et `addressStructured` à `null`.
  - **F4 (L)** : cycle de vie de l'avis, AC-6 bis.
  - **F5 (L)** : l'inventaire des écrans à version est donné, par sa commande.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
