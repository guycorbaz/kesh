# Story 23.1 : Le socle — deux gardes, parce qu'il y a deux silences

## Status

ready-for-dev

Story-zéro de l'**Epic 23 « Dette i18n »** (plan : `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`).
Elle ne résorbe presque rien — **20 clés sur 317**. Elle rend le reste *mesurable*, et surtout
elle rend impossible d'en ajouter d'autres sans le savoir.

## Story

**As a** personne qui utilise Kesh en allemand, en italien ou en anglais,
**I want** que le logiciel cesse de me servir du français en se taisant,
**so that** un oubli de traduction devienne un test rouge et non un état permanent.

Et, du côté de qui développe : **I want** qu'une clé écrite dans un `.svelte` et absente du
catalogue fasse échouer `npm run test:unit`, **so that** la dette cesse de se creuser pendant
qu'on la résorbe.

Adresse [#316] et [#283] — dont elle ne ferme aucune : elle les **borne**.

## Contexte — le défaut a deux chemins, pas un

C'est le point que toute rédaction rapide de cette story rate, et il commande tout le reste.

`I18nBundle::all_messages(locale)` (`crates/kesh-i18n/src/loader.rs:130-143`) **charge d'abord
toutes les clés `fr-CH` comme base**, puis écrase avec celles de la locale demandée. Le
frontend reçoit donc, par `GET /api/v1/i18n/messages`, un catalogue **déjà replié**. D'où deux
chemins distincts pour un même symptôme visible :

| | où la clé manque | ce que reçoit le frontend | quel repli s'affiche |
|---|---|---|---|
| **[#283]** — 57 clés | en `de-CH` / `it-CH` / `en-CH`, présente en `fr-CH` | **le texte français**, via le repli du loader | celui du **backend** — le 2ᵉ argument d'`i18nMsg` n'est jamais atteint |
| **[#316]** — 250 clés | **partout** | rien | le **littéral en dur** du `.svelte` |

⚠️ **Conséquence directe sur la conception des gardes** : aucun test passant par `format()` ou
par `all_messages()` ne peut voir [#283]. Ces deux fonctions rendent du français **exactement
comme si la traduction existait**. Le seul observable honnête est **l'ensemble des clés déclarées
dans chaque fichier `.ftl`** — ce que `I18nBundle` conserve déjà dans son champ `keys`
(`loader.rs:34`), peuplé par `fluent_syntax::parser` au chargement.

C'est aussi pourquoi les deux précédents du dépôt — `client_number_labels_are_translated_in_all_four_locales`
(16-3b) et `duplicate_probe_labels_are_translated_in_all_four_locales` (22-2b) — recourent à
`assert_ne!(msg, fr)` : faute d'accès aux ensembles de clés, ils déduisent l'absence du fait que
la locale rend le même texte que le français. **Cette ruse ne se généralise pas** : « Total »,
« CHF », « Journal », « Kesh » sont légitimement identiques en quatre langues, et un test
généralisé sur ce critère produirait des faux positifs en série.

## Périmètre

### Dedans

1. **Garde A — parité inter-locales**, dans `kesh-i18n` (Rust).
2. **Garde B — existence des clés demandées**, en vitest, portée `frontend/src` **entier**.
3. **Les 8 motifs dynamiques**, traités par énumération déclarée.
4. **Le moissonneur de replis**, versionné et exécutable.
5. **Le domaine pilote `contacts`** — 20 clés entrées aux quatre locales.

### Dehors, explicitement

- Les **297 autres clés** — elles sont le travail des stories 23-2 à 23-6.
- **[#255]** (chaînes en dur sans appel à `i18nMsg`). ⚠️ **C'est l'angle mort assumé des deux
  gardes** : ce qui n'est jamais demandé ne peut pas être trouvé manquant. La page `/invoices`
  en est le cas — **6 appels** `i18nMsg` pour toute la page.
- **[#314]**, le sélecteur de langue **[#242]**, et toute traduction des manuels LaTeX.
- **Le backend `kesh-qrbill`** et sa table `I18N_KEYS` / `DEFAULT_EN` (`types.rs:216`) : c'est un
  troisième catalogue, apparié par **position**, hors sujet ici.

## Un constat de la spécification — il y a 10 clés de plus qu'annoncé

L'énumération des motifs dynamiques, exigée par le point 3, a été **faite** pendant la
spécification plutôt que supposée. Sur les huit motifs, sept se résolvent en clés qui existent :

| motif | valeurs énumérées | présentes au catalogue |
|---|---|---|
| `journal-${j.toLowerCase()}` | 5 (achats, ventes, banque, caisse, od) | ✅ 5 |
| `account-type-${t.toLowerCase()}` | 4 | ✅ 4 |
| `due-dates-filter-${ps}` | 4 | ✅ 4 |
| `reports-filename-${type}` | 5 | ✅ 7 déclarées |
| `vat-category-${category}` / `${r.category}` | données de la table `vat_rates` | ✅ 5 |
| `reminders-error-${entry[0]}` | 15 + `-unknown` | ✅ 17 |
| **`imported-supplier-invoices-error-${entry[0]}`** | **10** | ❌ **0** |

**Le dernier motif est entièrement absent des quatre catalogues.** Ses dix valeurs
(`unsupported-file-type`, `file-too-large`, `symlink-rejected`, `duplicate`, `no-qr-code-found`,
`invalid-spc-payload`, `invalid-iban`, `pdf-render-error`, `file-read-error`, `field-too-long`,
carte à `supplier-invoices/import/+page.svelte:55-66`) ne figurent dans **aucun** décompte
antérieur, puisqu'elles ne sont jamais écrites comme littéraux.

**Le total de l'epic passe donc de 307 à 317**, et la story 23-3 de 99 à 109. Le plan d'epic est
corrigé en conséquence. ⚠️ *C'est exactement ce que le point 3 existe pour empêcher : un motif
dynamique que la garde ignorerait serait un trou **dans la garde elle-même**.*

## Décisions

**D1 — Garde A compare des ENSEMBLES DE CLÉS, jamais des textes rendus.**
Un test dans `crates/kesh-i18n/src/loader.rs` (module `tests`, accès au champ privé `keys`) ou,
si l'on préfère un test d'intégration, derrière un accesseur `pub fn keys(&self, locale: &Locale) -> &[String]`
ajouté à `I18nBundle`. **Le critère est l'égalité des quatre ensembles**, dans les deux sens :
une clé présente seulement en `de-CH` est un défaut au même titre (aujourd'hui il y en a **0**,
mesuré — la garde le maintient).
⚠️ **Ne PAS généraliser `assert_ne!(msg, fr)`** — motif à la § *Contexte*.

**D2 — Garde B généralise `duplicate-i18n-keys.test.ts`, elle ne le double pas.**
Le fichier `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts` (22-2b) est le
prototype : même façon de lire les `.ftl`, même parcours de `src`. La story le **remplace** par
un test de portée générale — `frontend/src/lib/shared/i18n-keys.test.ts` — et supprime l'ancien,
dont la borne `DOMAINE = /^contact-duplicate-/` n'a plus de raison d'être. **Un seul test, pas deux.**

**D2-bis — `contacts-i18n-realpath.test.ts` N'EST PAS supprimé.**
Le seul test retiré est `duplicate-i18n-keys.test.ts` (D2). ⚠️ L'autre test i18n de la 22-2b,
`frontend/src/routes/(app)/contacts/contacts-i18n-realpath.test.ts`, prouve **le chemin de rendu**
— le ternaire singulier/pluriel, avec le dictionnaire réel et sans mock d'`i18nMsg` — et non
l'existence des clés. Les deux gardes de cette story ne le recouvrent en rien : une clé peut
exister dans les quatre locales **et** être choisie à tort par le code qui la sélectionne.

**D3 — Les allowlists sont DÉCROISSANTES PAR CONSTRUCTION.**
Deux fichiers de données, `frontend/src/lib/shared/i18n-dette-connue.ts` (clés jamais entrées au
catalogue) et `crates/kesh-i18n/dette-parite-connue.txt` (les 57 de [#283]), triés, une clé par
ligne, chacun ouvert par un commentaire qui dit *ce que c'est* et *quelle story le vide*.
⚠️ **Chaque garde échoue AUSSI quand son allowlist contient une clé désormais présente** — sans
quoi l'allowlist se fossilise et la garde devient décorative. C'est la moitié qui compte : elle
force chaque story de rollout à retirer ses lignes.

**D4 — Les motifs dynamiques sont DÉCLARÉS, avec leurs valeurs, et bornés.**
Une table `MOTIFS_DYNAMIQUES` dans la garde B : `{ motif, valeurs[] }`. La garde vérifie chaque
`motif.replace(valeur)` comme une clé ordinaire.
⚠️ **Les valeurs sont écrites EN DUR dans le test, pas lues depuis le code de production.** Lire
la carte de production ferait qu'une carte vidée par erreur rendrait le test vert à vide — le
mode d'échec du **test muet**, déjà payé sur `backfill_skips_archived_accounts` (16-1a). En
contrepartie, chaque motif porte une assertion de **cardinalité attendue** (`assert_eq!` sur le
nombre de valeurs) qui rougit dès que la carte de production évolue sans le test.
⚠️ `vat-category-${category}` est le seul motif dont les valeurs sont des **données**
(catégories de `vat_rates`) et non un littéral : les cinq catégories seedées sont énumérées,
avec le commentaire qui le dit.

**D5 — Les deux gardes se bornent contre le passage à vide.**
Chacune assert un **minimum de clés effectivement collectées** (garde B : `>= 900` littéraux
demandés — 1010 mesurés ; garde A : `>= 1200` clés par locale — 1216 mesurés). Sans cette borne,
un motif de collecte cassé rend un test vert qui ne teste rien. *Le prototype 22-2b portait déjà
cette borne (`expect(demandees.size).toBeGreaterThanOrEqual(5)`) — elle est reprise et élargie.*

**D6 — Le moissonneur PROPOSE, il n'écrit jamais dans les catalogues.**
`frontend/scripts/harvest-i18n-fallbacks.mjs` — il lit `src`, extrait les couples
(clé, repli littéral), et rend un fragment `.ftl` **trié, sur la sortie standard**. Il ne touche
à aucun `messages.ftl`.
⚠️ **Motif** : un repli est écrit dans le feu de l'action, souvent sans majuscule, sans point
final, parfois avec la formulation d'un développeur pressé. **Le laisser devenir un libellé de
catalogue sans relecture, c'est faire entrer 250 approximations dans le produit.** Chaque story
de rollout relit ce qu'elle fait entrer.
⚠️ **Cinq clés n'ont PAS de repli littéral** — les cinq de `TransactionSplitModal.svelte`, dont
le repli est interpolé (`` `Ligne ${i + 1} : compte requis` ``). Le moissonneur les **liste à
part**, sur la sortie d'erreur, comme « à écrire à la main, entrée Fluent à variable ». Elles
sont traitées en **23-5**, pas ici.

**D7 — Le pilote est `contacts`, et il est choisi pour une raison.**
20 clés (12 sous `lib/features/contacts`, 8 sous `routes/(app)/contacts`). C'est le domaine que
la 22-2b vient de travailler, dont la garde bornée existe déjà, et dont le vocabulaire est
entièrement couvert par la **partie A** du glossaire — donc **aucune décision terminologique
n'est prise dans la story-zéro**.

**D8 — Le glossaire est un INPUT figé, et le pilote en tranche TROIS termes.**
`docs/i18n-glossaire.md` existe (kickoff du 2026-08-19). Sa **partie A est contraignante** : 48
équivalences relevées dans les 1216 clés déjà alignées, chacune nommant la clé qui l'atteste.
Sa **partie B attend l'arbitrage de Guy** — 15 termes sans précédent.

⚠️ **Correction d'une affirmation de la première rédaction de cette spec.** Elle disait
« aucun terme de la partie B n'apparaît dans les 20 clés du pilote (vérifié) ». **C'est faux, et
la vérification l'a montré** : `localité` (`field-city`, `contact-error-address-npa-city`),
`prénom` (`field-first-name`, `contact-persons-name-required`, `contact-error-person-name`) et
**`personne de contact`** — ce dernier n'étant même pas *dans* la partie B — y figurent, sur
**10 des 20 clés** — recompté. *Une affirmation portant le mot « vérifié » et qui ne l'était pas est
précisément ce que la § « Recompter ses propres comptes rendus » du `CLAUDE.md` vise.*

**Ce que cela change, et ce que cela ne change pas.** Les trois termes sont **sans enjeu
sémantique** — un champ d'adresse, un champ d'état civil, un libellé d'annuaire :
`Ort` / `località` / `city`, `Vorname` / `nome` / `first name`, `Kontaktperson` / `persona di
contatto` / `contact person`. La story les fixe et les **remonte en partie A** avec la clé qui
les atteste désormais. **L'arbitrage structurant — « analytique », centres de coûts contre
projets — reste intact et hors du pilote** : la story n'est donc pas bloquée, mais elle n'est pas
non plus neutre terminologiquement, ce qui était l'affirmation d'origine.

**`personne de contact` est à AJOUTER à la partie B** du glossaire dans cette story : il manquait.

**D8-bis — Le découpage par dossier ne fuit pas, et c'est mesuré.**
Sur les 250 clés manquantes, **2 seulement** sont demandées depuis plus d'un dossier —
`field-first-name` et `field-last-name`, et **les deux sont à l'intérieur du pilote**
(`features/contacts` et `routes/(app)/contacts`). Aucune clé n'est donc partagée entre deux
stories de rollout : chaque story peut vider sa part de l'allowlist sans coordination.

**D9 — Registre d'adresse, mesuré et non supposé.**
`de-CH` **vouvoie** (Sie-Form, 115 messages), `it-CH` **tutoie** (2ᵉ personne du singulier, 31
contre 1), `en-CH` reste à l'impératif neutre. Les 20 clés du pilote s'y conforment.

**D10 — Où les gardes s'exécutent, et pourquoi ça suffit.**
Garde A dans `cargo test -p kesh-i18n` (donc dans le gate backend et en CI). Garde B dans
`npm run test:unit` (donc dans le gate frontend et en CI). **Aucun nouveau script npm, aucune
nouvelle étape de CI** : les deux gates existants les portent. `lint-i18n-ownership` reste tel
quel — il répond à une autre question (l'appartenance d'un namespace à un dossier) et son
allowlist `KNOWN_VIOLATIONS` n'est pas touchée.

## Acceptance Criteria

1. **AC1** — Un test de `kesh-i18n` échoue si les quatre `messages.ftl` ne déclarent pas le
   **même ensemble** de clés, **allowlist de parité déduite**. Le message d'échec **nomme** les
   clés et la locale.
2. **AC2** — Ce test compare des **ensembles de clés**, sans jamais appeler `format()` ni
   `all_messages()`. *(Vérifiable : le corps du test ne mentionne ni l'une ni l'autre.)*
3. **AC3** — Le même test échoue si l'allowlist de parité contient une clé **désormais présente**
   dans les quatre locales.
4. **AC4** — Un test vitest échoue si une clé littérale passée à `i18nMsg()` **n'existe dans
   aucun** des quatre catalogues, allowlist de dette déduite. Le message nomme la clé **et le
   fichier** qui la demande.
5. **AC5** — Le même test échoue si l'allowlist de dette contient une clé désormais présente.
6. **AC6** — Le test vitest balaie **tout** `frontend/src` — `src/routes/` compris, où vivent
   **197 des 250** clés manquantes.
7. **AC7** — Les **8 motifs dynamiques** sont déclarés avec leurs valeurs et contrôlés comme des
   clés ordinaires ; chaque motif porte une assertion de cardinalité.
8. **AC8** — Les **10 clés `imported-supplier-invoices-error-*`** révélées par cette énumération
   figurent dans l'allowlist de dette avec le commentaire qui dit **quelle story les videra**
   (23-3).
9. **AC9** — Chaque garde porte une **borne de collecte minimale** (D5) qui la fait échouer si
   son motif d'extraction cesse de trouver quoi que ce soit.
10. **AC10** — `frontend/scripts/harvest-i18n-fallbacks.mjs` existe, rend un fragment `.ftl`
    trié sur la sortie standard, **ne modifie aucun fichier**, et liste séparément les clés sans
    repli littéral.
11. **AC11** — Les **20 clés du domaine `contacts`** existent dans les **quatre** locales, sont
    retirées de l'allowlist de dette, et leurs libellés `de-CH` / `it-CH` / `en-CH` respectent la
    partie A du glossaire et le registre de D9.
12. **AC12** — `duplicate-i18n-keys.test.ts` est **supprimé**, sa fonction étant reprise par la
    garde générale. *(Le contrôle qu'il exerçait ne doit pas disparaître : les quatre clés
    `contact-duplicate-*` restent couvertes, désormais par la garde B.)*
13. **AC13** — Les deux gates complets passent : backend (`cargo test --workspace`) et frontend
    (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`).

## Tasks / Subtasks

- [ ] **T1 — Garde A, parité inter-locales** (AC1, AC2, AC3, AC9)
  - [ ] Accesseur `keys()` sur `I18nBundle` si le test est hors module, sinon usage direct du champ privé
  - [ ] Comparaison des quatre ensembles dans les deux sens, message d'échec nommant clés et locale
  - [ ] `crates/kesh-i18n/dette-parite-connue.txt` — les 57 clés, triées, en-tête explicatif
  - [ ] Assertion « allowlist obsolète » + borne de collecte (`>= 1200` par locale)
- [ ] **T2 — Garde B, existence des clés demandées** (AC4, AC5, AC6, AC9, AC12)
  - [ ] `frontend/src/lib/shared/i18n-keys.test.ts` — parcours de tout `src`, lecture des 4 `.ftl`
  - [ ] `frontend/src/lib/shared/i18n-dette-connue.ts` — 250 + 10 clés, triées, en-tête qui nomme les stories de résorption
  - [ ] Assertion « allowlist obsolète » + borne de collecte (`>= 900` littéraux)
  - [ ] Suppression de `duplicate-i18n-keys.test.ts`
- [ ] **T3 — Motifs dynamiques** (AC7, AC8)
  - [ ] Table `MOTIFS_DYNAMIQUES` avec les 8 motifs et leurs valeurs en dur
  - [ ] Assertion de cardinalité par motif
  - [ ] Les 10 `imported-supplier-invoices-error-*` en allowlist, commentaire « résorbées par 23-3 »
- [ ] **T4 — Moissonneur** (AC10)
  - [ ] `frontend/scripts/harvest-i18n-fallbacks.mjs`, sortie standard uniquement
  - [ ] Clés sans repli littéral listées sur la sortie d'erreur
- [ ] **T5 — Pilote `contacts`** (AC11)
  - [ ] Moisson des 20 replis, **relecture** des libellés `fr-CH` avant de les figer
  - [ ] Traduction `de-CH` / `it-CH` / `en-CH` sur la partie A du glossaire, registre D9
  - [ ] Retrait des 20 clés de l'allowlist de dette
- [ ] **T6 — Gates** (AC13)
  - [ ] Gate backend complet, gate frontend complet, avant tout push

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas traduire au-delà des 20 clés du pilote.** Une story-zéro qui déborde n'est plus une
  story-zéro, et les rollouts perdent leur mesure.
- **Ne pas toucher `KNOWN_VIOLATIONS`** de `lint-i18n-ownership.js` : autre question, autre garde.
- **Ne pas « corriger » les 5 clés interpolées** de `TransactionSplitModal` — elles demandent des
  entrées Fluent à variables, et c'est la 23-5 qui les porte.
- **Ne pas ajouter de clé au catalogue sans l'ajouter aux QUATRE locales** : ce serait creuser
  [#283] dans la story qui vient la borner, et la garde A le refuserait de toute façon.

### Ce que le dev doit lire avant d'écrire

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-i18n/src/loader.rs:130-143` | `all_messages` et son repli `fr-CH` — la raison d'être de D1 |
| `crates/kesh-i18n/src/loader.rs:34` | le champ `keys`, seul observable honnête |
| `crates/kesh-i18n/src/loader.rs:261` et `:296` | les deux gardes bornées existantes (16-3b, 22-2b) et leur ruse `assert_ne!` |
| `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts` | le prototype à généraliser |
| `frontend/src/lib/shared/utils/i18n.svelte.ts:14-23` | `i18nMsg`, son repli, son interpolation `{ $var }` |
| `frontend/scripts/lint-i18n-ownership.js` | ce que l'autre lint fait — et ne fait pas |
| `docs/i18n-glossaire.md` | terminologie contraignante (partie A) et registre mesuré |

### Chiffres de référence — recomptés le 2026-08-19, à re-vérifier avant de les réécrire

| | valeur |
|---|---|
| clés `fr-CH` / `de-CH` / `en-CH` / `it-CH` | 1273 / 1216 / 1216 / 1216 |
| clés de [#283] | 57, **même ensemble** sur les trois locales, 0 clé en trop |
| littéraux demandés par `i18nMsg()` | 1010, dont **752 existent** |
| clés statiques manquantes | **250**, sur 13 dossiers, dont 197 sous `src/routes/` |
| clés révélées par les motifs dynamiques | **+10** (`imported-supplier-invoices-error-*`) |
| **total de l'epic** | **317** |
| replis moissonnables | 245 ; 5 interpolés |

Commande de recompte, à exécuter et non à croire :

```sh
cd crates/kesh-i18n/locales
comm -23 <(grep -oE '^[a-z0-9-]+ =' fr-CH/messages.ftl | sort -u) \
         <(grep -oE '^[a-z0-9-]+ =' de-CH/messages.ftl | sort -u) | wc -l
```

### References

- [Source: `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`] — plan d'epic, découpage, risques
- [Source: `docs/i18n-glossaire.md`] — terminologie contraignante, registre mesuré
- [Source: `crates/kesh-i18n/src/loader.rs`] — repli `fr-CH`, champ `keys`, gardes bornées existantes
- [Source: `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts`] — prototype de la garde B
- [Source: `CLAUDE.md` § *Recompter ses propres comptes rendus*] — tout décompte de cette story se recompte
- [Source: `CLAUDE.md` § *Migration breaking policy* P6] — précédent du **test muet**, motif de D4 et D5
- [#316] : https://github.com/guycorbaz/kesh/issues/316
- [#283] : https://github.com/guycorbaz/kesh/issues/283

## Change Log

### Création — 2026-08-19, Opus 5

Spec créée après le kickoff de l'epic. **Trois choses ont été établies pendant la spécification
plutôt que supposées**, et chacune a changé le contenu :

1. **Le défaut a deux chemins et non un** (`all_messages` replie sur `fr-CH`) — d'où D1, et le
   rejet explicite de la généralisation d'`assert_ne!(msg, fr)`.
2. **L'énumération des 8 motifs dynamiques révèle 10 clés manquantes de plus** — le total de
   l'epic passe de 307 à 317, la 23-3 de 99 à 109.
3. **Les 5 clés sans repli littéral sont toutes dans un seul composant** et demandent des entrées
   Fluent à variables — sorties du périmètre de la story-zéro, portées par la 23-5.

**Et une affirmation de cette même spec a été réfutée par sa propre vérification**, avant
relecture extérieure : D8 déclarait « aucun terme de la partie B dans le pilote (vérifié) »
alors que trois y figurent, sur 10 des 20 clés. Corrigé sur place, avec la mention de l'erreur —
*une correction n'est pas exempte du défaut qu'elle corrige.*

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
