# Story 34-6 : après un 409, l'édition de la société montre l'écriture concurrente avant de l'écraser (#953)

Status: review

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration, aucun changement serveur.** Registre :
`e34-choix-autonomes-e343.md`, C-e343-6, 7 et 14. Commits en `refs #953` ; à la PR : `closes #953`.*

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
- **Fusion à trois voies des champs non modifiés** (validation P2, 34-6-F2 ; C-e343-14). Après la relecture, chaque champ
  que l'Administrateur n'a pas modifié (`input[f] = base[f]`) prend la valeur relue (`input[f] := stored[f]`). Aucun travail
  n'est perdu, puisque rien n'y avait été saisi, et le second clic ne rétablit plus l'ancienne valeur sur l'écriture d'un
  collègue.
  - L'avis nomme ces champs « repris de l'enregistrement entre-temps : *valeur* », pour que la fusion se voie.
  - Sans la fusion, un collègue corrige la rue, l'Administrateur corrige le NPA, et son second clic remet l'ancienne rue :
    c'est le dommage de #953, déplacé du silence à un avertissement.
- **Sur un 409**, on relit, on fusionne, puis on compare les champs **du bloc** :
  1. **Aucun n'a changé** depuis la copie : le message actuel. L'essai suivant est sans danger. *(Au développement, LOW
     F1 de la validation P5 : ce cas lit `changed = ∅` **et** `merged = ∅` ; avec `merged ≠ ∅`, c'est l'avis de fusion du
     paragraphe suivant, et l'édition reste ouverte.)*
  2. **Tous égalent la saisie, après la fusion, et au moins un champ avait été modifié avant elle** (définition
     d'`already-saved`, ci-dessous) : l'édition se ferme, `data` porte la société relue, et une notification dit « déjà
     enregistré ».
  3. **Sinon** : sous chaque champ en conflit, « Enregistré entre-temps : *valeur* », la valeur vide se disant « (vide) ». Un
     avis dit qu'enregistrer de nouveau remplacera ces valeurs. La saisie modifiée reste.
- **L'avis de fusion s'affiche dans tout `kind` où `merged ≠ ∅`**, `unchanged` compris (validation P3, 34-6-F1). Le
  « message actuel » du cas 1 ne vaut que si `merged = ∅` **et** `changed = ∅`. Sinon le formulaire change sous les yeux de
  l'Administrateur sans le lui dire. Avec `already-saved`, l'édition se ferme, et la fusion n'a plus d'objet.
- **La copie prend les valeurs relues après toute relecture réussie**, quel que soit le cas, si bien qu'un second conflit se
  compare à elles (validation P2, 34-6-F5).
- **Les définitions, champ par champ** (validation P1, 34-6-F1) :
  - `merged = { f : input[f] = base[f] ∧ stored[f] ≠ base[f] }` : la saisie y prend `stored[f]` (fusion, P2) ;
  - `changed = { f : input[f] ≠ base[f] ∧ stored[f] ≠ base[f] ∧ stored[f] ≠ input[f] }`. Un champ modifié ailleurs **à la
    même valeur que la saisie** n'est pas en conflit : enregistrer ne le remplacera pas, et l'avis ne le nomme pas ;
  - `already-saved` si et seulement si `stored[f] = input[f]` pour tout champ du bloc, **après** la fusion, **et** qu'au
    moins un champ avait `input[f] ≠ base[f]` **avant** la fusion. Sans modification, rien n'a été enregistré par
    l'Administrateur, et le cas est `unchanged` (validation P3, 34-6-F2) ;
  - `unchanged` si `changed = ∅` et que le bloc n'est pas `already-saved` ;
  - `changed` sinon.
- **Une fonction pure** `companyEditConflict(base, stored, input, fields)` dans
  `frontend/src/lib/features/settings/company-conflict.ts`, qui rend `{ kind: 'unchanged' | 'already-saved' | 'changed',
  changed: string[], merged: string[], input: … }` (la saisie fusionnée), et **un composant** `CompanyConflictNotice.svelte`. Les trois blocs les emploient.
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
2. **AC-2 — aucun champ du bloc n'a changé.** L'Administrateur a modifié au moins un champ de l'adresse. Un 409 dû à un autre
   bloc (le téléphone modifié pendant l'édition) donne le message actuel, sans valeur affichée. L'essai suivant aboutit. Cet
   AC vaut quand `merged = ∅` et `changed = ∅`. Avec `merged ≠ ∅`, c'est l'AC-5 ter.
3. **AC-3 — déjà enregistré.** L'Administrateur a modifié au moins un champ, et la valeur relue égale sa saisie
   (définition d'`already-saved`) : l'édition se ferme sur « déjà enregistré ». Aucun second envoi. Sans modification de sa
   part, le cas n'est jamais `already-saved`.
4. **AC-4 — les trois blocs.** AC-1 à AC-3 valent pour l'e-mail (un champ), les coordonnées (deux champs, dont un seul peut
   avoir changé) et l'adresse (cinq composants).
5. **AC-5 — second conflit** (récrit à la validation P2, 34-6-F1). Premier conflit sur la ville (`Nyon` affiché). Puis une
   troisième écriture concurrente sur **la rue seule**. Au second 409, la rue est **reprise** (avis de fusion :
   l'Administrateur ne l'a pas modifiée), et **la ville n'est plus dans l'avis de conflit** (`kind = 'unchanged'`, `merged`
   contenant la rue). *(Reformulé à la validation P3, 34-6-F1 : la rue n'est pas « en conflit », elle est fusionnée.)*
   - Avec la copie relue, la ville est revenue à `stored = base`, si bien qu'elle n'est plus en conflit.
   - Avec l'ancienne copie, elle le serait encore.
   - C'est l'assertion que M2 fait rougir. Une troisième écriture sur la même ville ne la distinguerait pas.
5 bis. **AC-5 bis — champ égal à la saisie**, en deux montages (validation P2, 34-6-F7). Bloc coordonnées : l'Administrateur
   a saisi le téléphone `T` et le site `X`, tous deux différents de la copie.
   - **a)** L'autre écrit le site `X` seul : cas `unchanged`, aucun champ nommé.
   - **b)** L'autre écrit le site `X` et le téléphone `U` : seul le téléphone est nommé.

   L'avis ne nomme jamais le site.
5 ter. **AC-5 ter — fusion** (34-6-F2). Bloc adresse : l'Administrateur corrige le NPA, l'autre corrige la rue. Au 409, la
   rue du formulaire prend la valeur de l'autre, et l'avis la dit reprise. Le NPA saisi reste. Le second clic écrit **les
   deux**, et la rue de l'autre n'est pas rétablie à l'ancienne valeur.
6. **AC-6 — relecture impossible.** Le message `-conflict-reload-failed` reste, et la saisie aussi. Aucun avis de valeur.
6 bis. **AC-6 bis — cycle de vie de l'avis** (34-6-F4). L'avis de conflit et les valeurs affichées sont remis à zéro par
   `start*Edit`, par l'annulation de l'édition et par un enregistrement réussi. Un second conflit remplace l'avis, sans
   l'empiler.
7. **AC-7 — textes.** Les clés neuves (`settings-company-conflict-changed` avec la variable `$value`,
   `settings-company-conflict-merged` avec `$value`, `settings-company-conflict-will-replace`,
   `settings-company-conflict-already-saved` et `settings-company-conflict-empty` pour « (vide) », validation P2, 34-6-F4 ;
   noms à fixer au développement) aux
   **quatre** locales, avec un repli Svelte identique. `lint-i18n-ownership` et les gardes de replis
   (`i18n-repli-divergent-actif.test.ts`, `i18n-keys.test.ts`) restent verts, et leurs compteurs sont recomptés. Les variables
   Fluent passent par le troisième argument de `msg` (`+page.svelte:23-34`), jamais par un `.replace()`.
8. **AC-8 — manuel.** Une phrase là où le manuel décrit l'édition de la société : un enregistrement refusé parce qu'un autre
   a modifié la société montre ce qui a été enregistré entre-temps, et un second clic remplace. Le PDF est régénéré et
   contrôlé aplati.

## Tâches

- [x] **T1 — rouges d'abord**, journalisés dans `kesh-gate-logs/e343/rouges.log` :
  - Vitest de `company-conflict.ts` (rouge à l'import) ;
  - Vitest de `settings-page.test.ts` : le test `:289-310` récrit selon l'AC-1, et un test par bloc ;
  - E2E de l'AC-1.
- [x] **T2** : `company-conflict.ts` et ses tests. On couvre les trois cas, puis un champ égal à la saisie (AC-5 bis), vide
  contre `null`, les espaces, un pays en minuscules, un pays vide contre `CH`, et `addressStructured` à `null`. Puis la
  fusion (AC-5 ter), le cas `kind = 'unchanged'` avec `merged ≠ ∅`, la saisie sans modification (jamais `already-saved` ; avis
  de fusion seulement si `merged ≠ ∅`, édition laissée ouverte), et les deux montages de l'AC-5 bis.
- [x] **T3** : `CompanyConflictNotice.svelte`, les trois blocs, la copie prise à `start*Edit` et mise à jour après toute
  relecture réussie, la saisie fusionnée.
- [x] **T4** : i18n aux quatre locales, et les gardes de replis.
- [x] **T5** : E2E. `company-address.spec.ts:128-157` est récrit, et un cas « coordonnées, un seul champ changé » s'ajoute
  à `company-contact-details.spec.ts`.
- [x] **T6** : manuel et PDF ; captures de l'avis pour l'adresse et pour les coordonnées, en thème clair, dans
  `kesh-gate-logs/e343/captures/`.
- [x] **T7** : CHANGELOG *Corrigé* (`refs #953`).

### Mutations nommées

- **M1** : `companyEditConflict` compare à la saisie au lieu de la copie → rouge du cas « aucun champ du bloc n'a changé ».
- **M2** : la copie n'est pas mise à jour au conflit → rouge AC-5.
- **M3** : le cas `already-saved` envoie quand même → rouge AC-3 (envoi compté par le mock).
- **M4** : la comparaison sans `trim` → rouge du test « espaces ».
- **M5** : le bloc des coordonnées ne compare que le téléphone → rouge AC-4 (site changé des deux côtés, à des valeurs
  différentes).
- **M6** : la fusion retirée (un champ non modifié reste à sa valeur d'ouverture) → rouge AC-5 ter.

## Gates

Frontend entier (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; E2E complet au dernier commit de code. Aucun fichier
Rust n'est touché, hormis les `.ftl` : `cargo test -p kesh-i18n` et `textes_coherents` si une garde lit les clés.

## Ce qui ne se traite pas ici

- **Le verrou de période** (`submitLock`, `submitUnlock`, `:291` et `:306`) : il ne porte pas de saisie à écraser. Hors #953.
- **Les autres écrans à version** (validation P2, 34-6-F3). L'inventaire part du symptôme, non d'un nom de code. Commande :
  `git grep -l "OPTIMISTIC_LOCK_CONFLICT\|status === 409" -- frontend/src ':!*.test.ts'`, plus large que celle de la P1, qui
  ne lisait que `routes/`. Le traitement du 409 de chaque écran est lu, avec cette question : relit-il en gardant la saisie,
  de sorte qu'un second clic écrase sans montrer ?
  - **Oui, le symptôme de #953** : `routes/(app)/bank-accounts/+page.svelte:413-421`. Son doc-comment le dit : « le dialogue
    reste ouvert avec son message, et le second essai part avec la version relue ». **Non traité ici, issue demandée.**
  - **Non : la saisie est remplacée par la valeur relue** (perte de saisie, sans écrasement silencieux) :
    `settings/invoicing:256-262`, `settings/email-templates:171-177` (`syncDraftFromTemplate`, `:131-139`),
    `SessionIdleSection.svelte:75-81` (`apply`, `:29-32`), `JournalEntryForm.svelte:238-244`,
    `McpSettingsSection.svelte:91-97`, `settings/dunning:124-127,169-171` (`load`).
  - **Non : un dialogue de conflit, ou une fermeture** : `InvoiceForm.svelte:611-614`, `contacts:527-529,578-580`,
    `products:389-394,432-435`, `AutoBackupSection.svelte:192-193`.
  - **Non : « Rechargez la page », sans relecture**, où un second clic rend 409 de nouveau : `accounts:298,394,420`,
    `users:182`. Pas de saisie gardée : `settings/api-keys:223` (révocation) et `McpKeysSection.svelte:181`.
  - Les fiches de pièce (`invoices/[id]`, `supplier-invoices/[id]` et `/edit`) et les modules `.ts` de la liste (formats,
    API) sont hors de cette famille.

  ⚠️ **Lu au traitement du 409, non exécuté.** L'étape 0 du développement relit ce classement. Elle ferme l'ensemble par la
  même commande, et ajoute les formulaires qui envoient une `version` sans reconnaître le 409 (`git grep -ln "version:" --
  frontend/src ':!*.test.ts'`). Une issue s'ouvre par écran confirmé.

## Recoupements

- `settings/+page.svelte` : **34-11a1** y change l'affichage du compte bancaire (`:607`, ligne S16, AC-10,
  `34-11a1-compte-sans-iban.md:69`), dans un bloc distinct. Fusion par union (validation P2, 34-6-F6).
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
- 2026-11-10 : **validation P2 (Opus) : 3 M, 4 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p2-e343.md`. Prémisses relues :
  - le contrôle de version précède le no-op (`companies.rs:857-871`) ;
  - le traitement du 409 de chaque écran de l'inventaire ;
  - `34-11a1-compte-sans-iban.md:69`.
  - **La sévérité n'a pas baissé** : P1 1 M → P2 3 M. Signal de la règle de splitting, à déclarer au Project Lead, avec le
    constat de l'amendement D5 :
    - **F1 (M), neuf** : présent depuis la création, la M2 et l'AC-5 d'origine ne se tenaient pas l'un l'autre. AC-5 récrit.
    - **F2 (M), neuf** : préexistant, et conservé par les définitions de la P1 sans qu'elles l'aient créé. Le champ non
      modifié écrasait l'écriture du collègue. Fusion à trois voies, sur arbitrage de l'orchestrateur, consignée à
      C-e343-14 ; AC-5 ter, M6.
    - **F3 (M), né de la remédiation P1** (F5) : l'inventaire partait d'un nom de code. Il part maintenant du symptôme, et il
      est classé : un écran confirmé, `bank-accounts`, avec issue demandée.

    Deux défauts d'origine distincts et un défaut né d'une remédiation : pas de recyclage, la fiche reste dans
    `settings/`. Pas de découpage proposé.
  - **LOW** :
    - **F4** : clé « (vide) ».
    - **F5** : la copie est mise à jour après toute relecture.
    - **F6** : recoupement avec 34-11a1.
    - **F7** : AC-5 bis en deux montages.
- 2026-11-10 : **validation P3 (Sonnet) : 1 M, 1 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p3-e343.md`.
  - **F1 (M), né de la remédiation P2** (la fusion, C-e343-14). L'avis de fusion s'affiche dans tout `kind` où
    `merged ≠ ∅`. L'AC-2 est borné à `merged = ∅` et `changed = ∅`, l'AC-5 reformulé (la rue reprise, et non en conflit),
    et T2 gagne un cas. La sévérité baisse : P2 3 M, P3 1 M.
  - **F2 (L)** : `already-saved` exige au moins un champ modifié avant la fusion.
- 2026-11-10 : **validation P4 ciblée (Haiku) : 1 M, remédié** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p4-e343.md`. **F1 (M), né de la remédiation P3** (34-6-F2) : la précondition « au moins un
  champ modifié avant la fusion » est portée au cas 2 de « Sur un 409 » et à l'AC-3, et T2 est alignée sur la règle de
  l'avis de fusion.
- 2026-11-10 : **développement** (Opus 5.5, worktree `kesh-e346`, base `665de151`). Choix neufs au registre
  `e34-choix-autonomes-e346.md`, C-e346-1 à 7. Statut `review`.
  - **Étape 0.** LOW F1 de la validation P5 : traité par un membre de phrase au cas 1 de « Sur un 409 » (ce cas lit
    `changed = ∅` et `merged = ∅`). Lu au code : `+page.svelte` (base) `:78`, `:143`, `:203`, `data = await
    fetchCompanyCurrent()` puis la saisie gardée — la prémisse tient. Le manuel ne décrivait pas le conflit : `git grep -n
    "Conflit de version\|rechargées" -- 'docs/manual/*.tex'` ne rend rien. L'inventaire des écrans à version est relu par
    ses deux commandes (C-e346-7) : seul `bank-accounts` porte le symptôme.
  - **Écart à la consigne Edit** : les quinze tests ajoutés à la fin de `settings-page.test.ts` l'ont été par un `cat >>`
    (ajout en fin de fichier, sans remplacement) ; le reste du code par Edit ou Write (fichiers neufs). Les mutations, par
    un script à motif vérifié unique, restaurées par `git checkout --` et `touch`.
- 2026-11-10 : **revue de code P1 (Opus, trois lentilles) : 0 C, 0 H, 2 M, 12 L** (B : 1 M, 3 L ; E : 1 M, 3 L ; A : 1 M,
  6 L ; B-1 et E1-1 sont le même défaut, compté une fois dans les 2 M). Rapports `kesh-gate-logs/e346-review-p1-{B,E,A}.md`.
  Remédiation (Opus 5.5) : rouges `c8139a8c`, correctif `499e79d8`, PDF `1572872c`. Sort de chaque finding :
  - **B-1 = E1-1 (M), traité** : chaque édition ouverte envoie la version de sa copie, et n'adopte celle d'une société
    plus récente que si ses propres champs n'y ont pas bougé (`sameBlockValues`, `syncOpenBlocks` ; **C-e346-8**). Rouge
    d'abord : « deux éditions ouvertes » (le bloc coordonnées partait avec la version 9 relue par l'e-mail, au lieu de 1).
    Un test garde l'absence de conflit parasite après le succès d'un autre bloc. Mutations : M7 (envoyer
    `data.company.version`) et M8 (adopter sans condition) rougissent le premier test, M9 (n'adopter jamais) le second.
    Manuel : une phrase sur les blocs ouverts ensemble ; capture `e346-deux-blocs-ouverts.png`.
  - **A-1 (M), traité** : test AC-3 des coordonnées ; M3c (la M3 sur les coordonnées) le rougit.
  - **E1-2 et A-2 (L), traités** : un second conflit et une relecture impossible après un premier avis, pour l'e-mail et
    les coordonnées ; M2e, M2c (copie non mise à jour), M10e, M10c (avis non effacé) rougissent chacune son test.
  - **E1-3 (L), traité** : les champs sont inactifs pendant l'envoi et la relecture (**C-e346-9**) ; test, M11.
  - **B-3 (L), traité** : les testids des lignes sous les champs sont en kebab-case (`postal-code`) ; test, M12.
  - **B-4 (L), traité** : `emptyAddress()` pour les deux états de l'adresse, et des littéraux typés au lieu des
    `as CompanyJson`.
  - **A-4, A-5 (L), traités** : le manuel cite « Repris de l'enregistrement entre-temps », et décrit la condition du
    message « rechargées » sans la restreindre à un autre bloc.
  - **A-6 (L), traité** : Annuler puis Modifier remet l'avis à zéro, testé pour l'e-mail et les coordonnées (sans mutation
    propre).
  - **A-7 (L), traité** : l'AC-2 de l'e-mail vérifie le second envoi, avec la version relue.
  - **A-3 (L), traité** : ventilation des rouges corrigée par une annotation datée au Dev Agent Record.
  - **B-2 (L), en partie non traité** : l'avis qui reste à côté d'un refus 400 est une décision (C-e346-4), maintenue. Les
    lignes sous les champs qui ne suivent pas la saisie modifiée après le 409 : **issue demandée** (P4).
  - **E1-4 (L), non traité, issue demandée** (P4) : `trim()` de JavaScript et `str::trim` de Rust ne retirent pas les
    mêmes caractères (U+0085, U+FEFF). Il faudrait aligner la normalisation sur le serveur, hors de cet écran.
  - *(Annoté le 2026-11-10, revue P2 A2-2 : les issues ouvertes par l'orchestrateur sont **#991** pour B-2 et **#992**
    pour E1-4. Celle de l'écran `bank-accounts` est **#984**.)*

  Gates au dernier commit de code `499e79d8` : `npm run check` 0 erreur, 29 avertissements ; `lint-i18n-ownership` PASS ;
  `test:unit` 223 fichiers, 2928 tests verts (2915 + 13) ; `build` vert ; E2E complet r4 (dossiers au runner), 386 passés,
  7 échecs (les sept de la KF-029), 18 ignorés ; `make user` et PDF contrôlé aplati. Ni Rust ni `.ftl` touchés :
  `test-fast` non relancé. Le prompt de la passe P2 est `34-6-societe-apres-409-review-prompt-p2.md`.
- 2026-11-11 : **revue de code P2 (Sonnet, trois lentilles) : 0 C, 0 H, 1 M, 13 L** (B : 1 M, 5 L ; E : 1 M, 3 L ; A :
  5 L ; P2-B1, E2-1 et A2-5 sont le même défaut, compté une fois). Rapports `kesh-gate-logs/e346-review-p2-{B,E,A}.md`.
  Remédiation (Opus 5.5) : rouges `fd354737`, correctif `8ee97e5e`. Sort de chaque finding :
  - **P2-B1 = E2-1 = A2-5 (M), traité** (**C-e346-10**). Toute société reçue passe par `adoptCompany`, qui ignore une
    société plus ancienne que `data`. `syncOpenBlocks` n'adopte qu'une version strictement supérieure. Le bloc en 409
    garde sa logique.
    - Rouge d'abord, dans l'ordre relecture en vol, succès d'un autre bloc, retour de la relecture : l'écran revenait à
      l'ancienne valeur.
    - M13 (garde de `adoptCompany` retirée) et M16 (adresse non adoptée) le font rougir.
    - M14 (garde de `syncOpenBlocks` retirée) est **équivalente** : la version d'un bloc ne dépasse jamais celle de
      `data`, si bien que la garde de `adoptCompany` suffit.
  - **A2-4 (L), traité** : la branche adresse de `syncOpenBlocks` est couverte par le test de monotonie (M16) et par un
    test « adresse ouverte, champs bougés » (M15).
  - **E2-3 (L), traité** : le verrouillage et le déverrouillage passent par `adoptCompany`. Les éditions ouvertes
    inchangées adoptent la société rendue : test rouge d'abord, M17.
  - **E2-2 (L), traité** : test « adresse relue absente ».
  - **B2 (L), traité** : `readonly` au lieu de `disabled` sur les huit champs, pour qu'ils gardent le focus (C-e346-9
    amendé) ; test rouge d'abord, M18.
  - **B3 et E2-4 (L), traités** pour la région vive : `role="status"` et `aria-live="polite"` sont montés à l'ouverture
    de l'édition, et l'avis s'y insère ; test, M19. **Non traité, issue demandée** (P4) : relier les lignes sous les
    champs à leur champ par `aria-describedby`.
  - **B4 (L), non traité, issue demandée** (P3) : `--color-warning` `#d97706` sur blanc mesure **3,19:1** (formule WCAG
    de luminance relative), sous les 4,5:1 requis pour un texte de 12 px. Le jeton est partagé : 11 usages de
    `text-warning` dans `frontend/src`.
  - **B5 (L), traité** : l'avis du bloc e-mail passe sous la ligne entière (`max-w-md`), et n'est plus plaqué à la
    largeur du champ.
  - **B6 (L), traité** : indentation des attributs.
  - **A2-1, A2-2 (L), traités** : annotations datées au Dev Agent Record (décomptes à `8ee97e5e`, mutations des
    remédiations, File List) et à l'entrée P1 (#984, #991, #992), sans réécrire les entrées passées.
  - **A2-3 (L), traité** : l'écart est dit au journal des rouges (`rouges.log`, section P2).

  **Une pollution de tests sous mutation** : sous M17, le test de la région vive rougit aussi. La valeur
  `mockResolvedValueOnce` du test du verrou n'est pas consommée quand celui-ci échoue, et `vi.clearAllMocks` ne vide pas
  ces files. Ce n'est vu que sous mutation : trois exécutions sans mutation sont vertes.

  **Gates au dernier commit de code `8ee97e5e`** (une session mybibli partageait la station ; `free -g` relevé avant
  chaque prise) :
  - `npm run check` : 0 erreur, 29 avertissements ; `lint-i18n-ownership` : PASS ;
  - `test:unit` : 223 fichiers, 2933 tests verts (2928 + 5) ; `build` : vert ;
  - E2E complet r5 (dossiers au runner) : 386 passés, 7 échecs, les sept de la KF-029 (`mode-expert:26`, `:41`,
    `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`), 18 ignorés.

  Ni Rust, ni `.ftl`, ni manuel ne sont touchés : `test-fast` et `make` ne sont pas relancés. **Captures différées** par
  la pause des commandes lourdes (`PAUSE-HEAVY`, décision de Guy du 2026-11-11, priorité à la release de mybibli) : elles
  sont rejouées à la levée de la pause. Le prompt de la passe P3 est `34-6-societe-apres-409-review-prompt-p3.md`.
  - *(Annoté le 2026-11-11, revue P3 A3-4 et A3-5. Les issues demandées ci-dessus sont **#997**, pour `aria-describedby`,
    et **#996**, pour le contraste. Le décompte de cette entrée est faux d'un LOW : l'A2-5 est absorbé par le MEDIUM
    commun, si bien que la passe rend **12 L**, et non 13.)*
- 2026-11-11 : **revue de code P3 (Opus, trois lentilles) : 0 C, 0 H, 1 M, 13 L.**
  - B rend 1 M et 6 L ; E rend 0 M et 4 L ; A rend 1 M et 5 L, soit 2 M et 15 L bruts.
  - Le MEDIUM (P3-B1 = A3-1) est un seul défaut : E le classe LOW (E3-1), et A3-2 en porte le corollaire. Il est compté une
    fois, sous M.
  - Les LOW communs comptent une fois : P3-B3 = E3-3 = A3-3, et P3-B6 = E3-4.
  - Rapports : `kesh-gate-logs/e346-review-p3-{B,E,A}.md`.

  Remédiation (Opus 5.5), faite pendant la pause des commandes lourdes : rouges `8444ca86`, correctif `89397483`.

  Sort de chaque finding :
  - **P3-B1 = A3-1 (M), avec E3-1 et le premier cas de A3-2 : traité** (**C-e346-11**).
    - Le correctif : une séquence client est attribuée au départ de chaque requête. À version égale, `adoptCompany`
      n'adopte que la réponse d'une requête partie après celle dont vient `data`.
    - Le test `:898` est corrigé : il dit que la version 5 vient de l'écriture d'un autre, et le modèle porte cette
      écriture.
    - Deux tests neufs jouent le vrai serveur, un verrou à version égale :
      - « verrouillage à version égale » : l'écran montre la borne. Il était vert avant, et la mutation `<` → `<=`
        d'`adoptCompany` doit le faire rougir.
      - « une relecture partie avant le verrou… » : rouge d'abord.
    - Limite assumée et écrite à C-e346-11 : la séquence ordonne les départs, non le traitement au serveur.
  - **P3-B2 (L), traité** par la même séquence : une relecture refusée parce que plus ancienne juge le bloc contre
    `data` ; rouge d'abord.
  - **E3-2 (L) et le second cas de A3-2** (une version qui baisse après une restauration) : non traités, **issue
    demandée** (P4). Adopter son propre succès quoi qu'il arrive ferait reculer l'écran quand deux succès se croisent.
  - **P3-B3 = E3-3 = A3-3 (L), traité** : `vi.resetAllMocks()` remplace `vi.clearAllMocks()` dans les six `beforeEach`
    de `settings-page.test.ts`. Aucun test n'a rougi du changement.
  - **P3-B6 = E3-4 (L), traité** : l'avis se vide au départ de l'envoi (**C-e346-4 renversé**) ; rouge d'abord.
  - **P3-B5 (L), traité** : `read-only:bg-input/50 read-only:cursor-progress` sur les huit champs. Le style est posé
    dans la page, le composant `Input` étant partagé : d'autres écrans emploient `readonly` à demeure.
  - **P3-B7 (L), traité** : le titre du test dit « pendant l'envoi ».
  - **P3-B4 (L)** : avec A3-1.
  - **A3-4, A3-5 (L), traités** : annotation datée de l'entrée P2.
  - **A3-6 (L), traité** : l'écart est dit au `rouges.log`, section P3.

  **Différé par `PAUSE-HEAVY`** (toujours en place), à jouer à sa levée, au dernier commit de code `89397483` :
  - les mutations, dont `<` → `<=` d'`adoptCompany` et la garde de séquence ;
  - Vitest complet, `npm run check`, `lint-i18n-ownership`, `build` et l'E2E complet ;
  - les captures, déjà différées en P2.

  Seuls les deux fichiers de test de la story ont tourné, isolés sous `mem-guard`, comme la pause le permet : 74 verts.
  Le manuel ne change pas. Le prompt de la passe P4 est `34-6-societe-apres-409-review-prompt-p4.md`.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (claude-opus-5-5), en autonomie.

### Completion Notes List

- **Rouges d'abord** (`kesh-gate-logs/e346/rouges.log`) : Vitest, `company-conflict.test.ts` rouge à l'import et 12 rouges
  de page (les AC-2 et AC-5 bis a) étaient verts avant le correctif : ils fixent le message actuel) ; E2E sur le frontend
  de base, 3 rouges (AC-1 adresse, conflit des coordonnées, AC-4), éléments introuvables. *(Annoté le 2026-11-10, revue
  P1 A-3 : les verts d'avant le correctif étaient quatre, et non trois. Ce sont l'AC-2 de l'adresse, l'AC-2 de l'e-mail,
  l'AC-5 bis a) et l'AC-6. Le total de 12 rouges reste juste.)*
- **Mutations** (`kesh-gate-logs/e346/mut/M1..M6.log`), toutes rouges : M1 9 tests, M2 1 (AC-5 de page), M3 1 (AC-3),
  M4 3, M5 1 (AC-4), M6 7. Aucune équivalente. Le fichier restauré n'a laissé aucune modification (`git status`).
  *(Annoté le 2026-11-10, revue P2 A2-1 : les mutations des remédiations sont au même dossier. En P1 : M7 à M12, M2e,
  M2c, M3c, M10e et M10c. En P2 : M13 à M19, dont M14 équivalente. Leur effet est donné à chaque entrée du Change Log.)*
- **Tests**, recomptés à la source de `665de151` à `HEAD` : `company-conflict.test.ts` 0 → 19 `it(` ;
  `settings-page.test.ts` 16 → 31 `it(` (+15, et celui de `:289-310` récrit) ; E2E `company-address.spec.ts` 4 → 4 (un
  récrit), `company-contact-details.spec.ts` 5 → 6 (le test 4 récrit, un 4 bis ajouté). *(Annoté le 2026-11-10, revue
  P2 A2-1 : ces chiffres valent à `e549552f`, fin du développement, et non à `HEAD`. Recompté à la source par
  `/usr/bin/grep -cE '^\s*(it|test)\('`, de `665de151` à `8ee97e5e` (dernier commit de code de la remédiation P2) :
  `company-conflict.test.ts` 0 → 21 ; `settings-page.test.ts` 16 → 44. Les E2E sont inchangés. Les tests exécutés et
  les gates de chaque remédiation sont aux entrées « revue de code P1 » et « revue de code P2 » du Change Log.)*
- **i18n** : six clés `settings-company-conflict-*` (`changed`, `merged`, `will-replace`, `merged-notice`,
  `already-saved`, `empty`) aux quatre locales (`/usr/bin/grep -c "^settings-company-conflict-"` : 6 par locale), repli
  identique ; `sitesTotal` 2922 → 2928 (+6, ventilé dans `i18n-keys.test.ts`).
- **Gates au dernier commit de code** `91a2acf0` :
  - `npm run check` : 0 erreur, 29 avertissements (30 avec celui que la story introduisait, corrigé par `91a2acf0`) ;
    `lint-i18n-ownership` PASS ; `test:unit` 223 fichiers, 2915 tests verts ; `build` vert.
  - Bases remises à neuf, `scripts/test-fast.sh --ci` : 5138 exécutés, 5138 passés, 12 ignorés (320 s) ;
    `cargo build --workspace --all-targets` vert.
  - `scripts/gate-ci.sh` **non lancé** : aucun fichier Rust touché ; les `.ftl` sont couverts par `test-fast` (parité
    des locales, `textes_coherents`).
  - **E2E complet**, base `kesh_e2e_e346` recréée vide, frontend construit à `91a2acf0` : run r3, **386 passés, 7 échecs,
    18 ignorés** (17,6 min). Les sept sont ceux de la KF-029 (#97) nommés par `docs/testing.md` : `mode-expert:26`,
    `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`. Les dix tests de `company-address` et
    `company-contact-details` passent. Un run r2 antérieur (369 passés, 13 échecs, 11 non joués) est **écarté** : le
    runner n'y recevait pas `KESH_INBOX_DIR` ni `KESH_DOCUMENTS_DIR`, et les six échecs hors KF-029 le disaient
    (« répertoire absent ou non inscriptible par le runner », `ENOENT … /tmp/kesh-e2e/inbox`) ; r3 les passe au runner.
- **Manuel** : § *Vos coordonnées sur la facture* du manuel utilisateur, PDF régénéré (`make user`, 3 passes) et contrôlé
  aplati (`pdftotext … | tr '\n' ' '`, la phrase « Si quelqu'un a modifié la société entre-temps… » s'y lit entière).
- **Captures** (thème clair, `kesh-gate-logs/e346/captures/`) : `e346-adresse-conflit-et-fusion.png`,
  `e346-coordonnees-un-champ-en-conflit.png`, `e346-email-vide-ailleurs.png`, `e346-coordonnees-autre-bloc.png`.
- **Issue à ouvrir** : `bank-accounts/+page.svelte:413-421`, le second essai part avec la version relue sans montrer
  l'écriture concurrente (symptôme de #953 sur un autre écran). *(Ouverte par l'orchestrateur : #984.)*

### File List

- `frontend/src/lib/features/settings/company-conflict.ts` (neuf)
- `frontend/src/lib/features/settings/company-conflict.test.ts` (neuf)
- `frontend/src/lib/features/settings/CompanyConflictNotice.svelte` (neuf)
- `frontend/src/routes/(app)/settings/+page.svelte`
- `frontend/src/routes/(app)/settings/settings-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/company-address.spec.ts`
- `frontend/tests/e2e/company-contact-details.spec.ts`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/34-6-societe-apres-409.md`, `34-6-societe-apres-409-review-prompt-p1.md`,
  `34-6-societe-apres-409-review-prompt-p2.md`, `34-6-societe-apres-409-review-prompt-p3.md`,
  `34-6-societe-apres-409-review-prompt-p4.md`,
  `e34-choix-autonomes-e346.md`, `sprint-status.yaml`
