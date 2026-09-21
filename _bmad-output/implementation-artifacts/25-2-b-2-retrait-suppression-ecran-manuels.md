# Story 25.2-b-2 : La suppression ne traite plus que les brouillons — l'écran et les manuels

Status: ready-for-dev

**Issue : [#440]**, qu'elle **ferme** : `closes #440` dans le **titre ET le corps** de sa PR.
⚠️ Elle **ne rouvre pas [#219]**, fermée : la story en remplace le chemin, pas la décision. Et elle
**ne ferme pas [#381]**, close par la 25-2-c.

⛔ **Issue du DÉCOUPAGE de la 25-2-b** (arbitrage de Guy, 2026-09-21). **La fiche mère
`25-2-b-devalidation-facture.md` reste la source des faits établis** par ses trois passes de
validation.

**Prérequis** : la **25-2-b-1** mergée — elle pose `unvalidate`, sans quoi retirer la suppression
d'une facture validée laisserait les utilisateurs sans aucun chemin de correction.

## Story

En tant que **personne qui facture**,
je veux que **« supprimer » ne concerne plus qu'un brouillon**, et que l'écran m'offre
**« dévalider »** sur une facture validée,
afin que **la destruction d'une écriture ne soit plus un effet de bord caché du verbe
« supprimer »**.

## Acceptance Criteria

1. **`invoices::delete` ne traite plus que les brouillons.** Sa branche `status == "validated"` et
   ses trois gardes disparaissent — elles vivent désormais dans `unvalidate`. Une facture validée y
   est refusée sous un code propre — **`INVOICE_MUST_BE_UNVALIDATED_FIRST`, `409`** — dont le
   message **oriente vers la dévalidation**, et non sous le générique `ILLEGAL_STATE_TRANSITION`.
   ⚠️ **Aucun test neuf n'est écrit pour cela** : ce sont les **deux tests retargetés** de l'AC 3 qui
   l'exercent — l'un au dépôt, l'autre à la couche HTTP. *(La rédaction précédente en prescrivait un
   troisième, doublon des deux.)* La branche retirée ne doit pas redevenir un chemin silencieux.

2. **`enforce_immutability = false` n'a plus qu'un appelant de production** — `unvalidate`. Le
   drapeau **reste dans `delete_in_tx`**, jamais chez l'appelant : une garde posée chez l'appelant
   laisserait la fonction nue pour le suivant. ⚠️ **Les trois doc-comments qui nomment
   `invoices::delete` comme seul appelant** (`journal_entries.rs:943`, `:982-989` ;
   `invoices.rs:1339-1340`) sont réécrits **par la 25-2-b-1**, qui les périme en créant le second
   appelant. ⛔ **NEUF AUTRES sites affirment encore que `invoices::delete` détruit l'écriture d'une
   facture validée**, et ils deviennent faux **ici** — aucun test ne les garde. Ventilation, parce
   qu'un total doit être cohérent avec la sienne : 1 (`journal_entries.rs:970`) + 2 (`lib.rs`) +
   1 (`routes/invoices.rs`) + 2 (`invoice_delete_e2e.rs`) + 3 (`+page.svelte`) = **9**, auxquels
   s'ajoutent les **deux** déjà nommés ci-dessus, qui se réécrivent ici une **seconde** fois —
   **onze ancrages en tout**. *(« Huit » était faux : cinquième décompte pris en défaut dans cette
   story.)*
   - *(les deux déjà nommés)* `journal_entries.rs:943` et `:982-989` — **deuxième réécriture** (le paragraphe court jusqu'à `:989`, « le résidu est assumé et tracé ») : b-1 les fait dire « deux
     appelants », b-2 en retire `invoices::delete` **et** sa justification (« sous ses trois gardes
     propres »), qui déménage chez `unvalidate` ;
   - `journal_entries.rs:970` — « (ex. `invoices::delete` d'une facture validée — #219) » ;
   - `crates/kesh-api/src/lib.rs:234` et `:416` ;
   - `crates/kesh-api/src/routes/invoices.rs:5` (doc de module) ;
   - `crates/kesh-api/tests/invoice_delete_e2e.rs:1` et `:6` (doc de module) ;
   - `frontend/src/routes/(app)/invoices/[id]/+page.svelte:68`, `:95` — et `:920`, qui **part avec
     le bloc retiré** par l'AC 4 (`:919-943`) au lieu de se réécrire : il compte parmi les onze
     ancrages, non parmi les gestes de réécriture.
   ⛔ **Un DIXIÈME site existe, et il NE SE TOUCHE PAS** :
   `crates/kesh-db/migrations/20260715000001_invoice_reminders.sql:14` — « FK `invoice_id ON DELETE
   CASCADE` : aligné sur la **suppression définitive** #219 ». **P8** interdit de modifier une
   migration appliquée, *pas même un commentaire* : `sqlx` en compare le checksum et **le binaire ne
   boote plus**. ⚠️ Le gate backend ne verrait rien (bases éphémères) ; seule l'E2E rougirait, après
   coup — précédent Story 16-3b. La remarque y reste, périmée et inoffensive. *Un grep du symptôme
   y mène droit : c'est écrit ici pour que personne ne « corrige » ce fichier.*

   Ceux du bras `Some(inv) if inv.status == "validated"` (`invoices.rs:1236-1282`) **avec son
   commentaire d'en-tête** (`:1232-1235`), plus `:1331` et **`:1339-1349`**, **partent avec la
   branche** : ils ne se réécrivent pas. ⛔ **Et c'est le BLOC `invoices.rs:1337-1354` qui part en
   entier**, pas seulement ses commentaires : c'est lui qui porte le `false`
   (`delete_in_tx(&mut tx, company_id, je_id, user_id, false)`, `:1350`), et il vit **hors** du
   `match` que l'AC 1 retire. Le laisser rendrait l'AC 2 **faux en silence** — un brouillon n'ayant
   pas de `journal_entry_id`, le `if let` ne se déclencherait jamais et **rien ne rougirait**.
   ⚠️ *Trois citations successives ont été fausses ici :
   `1228-1231` visait le bras `NotFound` et le bras **brouillon** (qui reste), `1232-1282` désignait
   le bras « en entier » alors qu'il commence à `1236`, et `1339-1345` coupait un commentaire qui
   court jusqu'à `1349`.*

3. **HUIT tests existants changent d'objet, et PAS tous de la même façon** — relevé en passe 2 :
   **quatre** d'entre eux (motifs 1, 2, 3, 5) réaffirmeraient mot pour mot ce que la **25-2-b-1**
   teste déjà sur `unvalidate` — ⚠️ **le motif 6 fait exception**, cf. sa ligne du tableau. La
   consigne uniforme « réécrire contre `unvalidate` » produirait des doublons ; ⛔ **ce qui n'est
   couvert nulle part, c'est le refus neuf** de `delete` sur une facture validée
   (`INVOICE_MUST_BE_UNVALIDATED_FIRST`), pour lequel l'AC 1 ne prescrit **aucun test neuf** mais
   **les deux tests retargetés ci-dessous** — l'un au dépôt, l'autre à la couche HTTP. Chaque
   test est donc traité **nommément**, et non par une règle à appliquer au jugé. ⚠️ **Quatre**
   d'entre eux seulement (motifs 1, 2, 3, 5) feraient doublon avec la b-1 ; **le motif 6 fait
   exception**, cf. sa ligne du tableau :

   | Test | Ce qu'il devient | Pourquoi |
   |---|---|---|
   | `test_delete_validated_unpaid_open_fy_removes_invoice_and_je` | **retargeté** sur le refus `INVOICE_MUST_BE_UNVALIDATED_FIRST`, **et RENOMMÉ** (p. ex. `test_delete_validated_requires_unvalidate_first`) | c'est le test du **succès** qui disparaît : après l'AC 1, aucun succès n'existe plus sur ce chemin. ⛔ **Le renommage n'est pas cosmétique** : `…_removes_invoice_and_je` affirmerait un résultat que le test ne mesure plus — c'est la famille du test muet |
   | `delete_validated_as_admin_returns_204` | **retargeté** sur le même refus par l'API (`409`), **et RENOMMÉ** (p. ex. `delete_validated_requires_unvalidate_first`) | idem, côté HTTP — son nom porte un code de retour qu'il n'aura plus |
   | `…_paid_is_rejected`, `…_credited_by_avoir_…`, `…_with_reminders_…`, `…_in_closed_fy_…` | **supprimés ici** (motifs 1, 2, 3, 5), leur propriété étant reprise par les tests d'empêchement de la **25-2-b-1** | réécrits contre `unvalidate`, ils **répéteraient mot pour mot** ceux de b-1 ; ⛔ **ne les supprimer qu'après avoir vérifié nommément que le test de b-1 existe** pour chaque motif — contrôlé en passe 4 : les quatre contreparties sont prescrites (b-1, AC 6 et AC 11) |
   | `delete_validated_in_locked_period_returns_400_period_locked` | **RÉÉCRIT par la dévalidation** (motif 6, couche HTTP) — et **non supprimé** | ⛔ *Le tableau et la liste qui le suit se contredisaient sur ce test, trois passes durant.* Il est le **seul site de bout en bout du chemin facture** pour le verrou de période ; la b-1 prescrit ce motif au **dépôt**, et `period_lock_e2e.rs:339,344` ne couvre que l'écriture nue |
   | `invoices.spec.ts:333-370` (Playwright) | **adapté** : dévalide d'abord, puis supprime le brouillon | le parcours qu'il mesure — « fiche fantôme » — ne change pas, son chemin si |

   Aucun ne disparaît sans que sa propriété soit reprise ailleurs :
   - `crates/kesh-db/src/repositories/invoices.rs` : `test_delete_validated_unpaid_open_fy_removes_invoice_and_je`,
     `test_delete_validated_paid_is_rejected`, `test_delete_validated_in_closed_fy_is_rejected`,
     `test_delete_validated_credited_by_avoir_is_rejected`,
     `test_delete_validated_with_reminders_is_rejected` ;
   - `crates/kesh-api/tests/invoice_delete_e2e.rs` : `delete_validated_as_admin_returns_204`, et
     ⛔ **`delete_validated_in_locked_period_returns_400_period_locked`**, ajouté par la
     **25-2-b-zero** (#443) — relevé en passe 3, il porte le motif « période verrouillée » par le
     chemin de suppression ; il se réécrit par la dévalidation.
   - Et le Playwright `frontend/tests/e2e/invoices.spec.ts:333-370` (« fiche fantôme »), qui
     supprime une facture **validée** par l'API : il dévalide d'abord.
   ⚠️ **HUIT au total — 5 + 2 + 1 —, et ce chiffre a été faux trois fois.** La fiche mère disait
   « cinq » (elle ignorait les deux E2E backend), cette fiche a d'abord titré « six » en en
   énumérant huit, et sa propre note de correction disait « sept ». Recompté depuis la source en
   passe 1 de validation : `grep -c "async fn test_delete_validated" invoices.rs` → **5** ;
   `invoice_delete_e2e.rs` → **2** (`:203`, `:246`) ; Playwright → **1**. ⛔ *Corriger un décompte
   et écrire juste sont deux gestes distincts.* ⚠️ `delete_requires_auth_returns_401` et
   `delete_as_comptable_returns_403` sont **exclus à bon droit** : ils échouent en amont, sur l'auth
   et le rôle, sans atteindre la branche retirée.

4. **L'écran.** Sur une facture **validée**, le bouton « Supprimer » devient **« Dévalider »**, et
   dit ce qui va se passer : l'écriture comptable est supprimée, la facture repasse en brouillon,
   son numéro est conservé. Les refus de l'AC 3 de la fiche mère s'affichent **en nommant leur
   motif**. Friction : une confirmation simple — **pas** de numéro à retaper, la facture restant en
   place.
   - ⚠️ **Résidu à retirer, et PAS « le bloc en entier »** (relevé en passe 2) : dans
     `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, retirer le test
     `{#if invoice?.status === 'validated'}` (`:919`) et son corps (`:920-942`), en **conservant**
     le contenu de la branche `{:else}` (`:944`, la confirmation du brouillon). ⚠️ **Retirer `:919`
     à `:943` ET `:945`** — sinon le `{:else}` et le `{/if}` restent orphelins et `svelte-check`
     échoue. ⚠️ Ce contenu **ne devient pas inconditionnel** : il change de condition — du statut
     vers la **présence d'un numéro** (AC 5). Deviennent morts aussi
     `let deleteConfirmText = $state('')` (`:97`, et son commentaire `:95`), sa remise à zéro dans
     `onOpenChange` (`:911`), la condition `:956-958`, **et l'import `Input` (`:3`)**, dont le seul
     usage du fichier est à `:936`, dans le corps retiré — `npm run check` rougirait.
   - ⛔ **La garde de `:729` change de rôle, et c'est un ÉLARGISSEMENT** (arbitrage de Guy du
     2026-09-19, perdu entre les deux filles et rétabli en passe 2) : `isAdmin && !invoice.paidAt`
     devient la **garde comptable** (`canManage`, `:65-67`) — jusqu'ici, seul l'Administrateur
     pouvait faire disparaître l'écriture d'une facture. Laissée telle quelle, **le Comptable ne
     verrait jamais le bouton « Dévalider »** et l'arbitrage ne serait pas livré.
   - ⚠️ **Et `!invoice.paidAt` sous-couvre le motif 1** : une facture *partiellement* réglée n'a pas
     de `paid_at`. L'écran lit le résiduel, ou assume le refus serveur — au choix, mais écrit.
   - ⚠️ **L'asymétrie des deux sorties s'affiche** (arbitrage de Guy, 2026-09-19) : dévaliser est
     ouvert au **Comptable**, mais **effacer** reste réservé à l'**Administrateur** (`admin_routes`,
     décision de #219, fermée aux clés API). ⛔ **Le `403` vient du bouton SUPPRIMER de la branche
     brouillon** — `+page.svelte:626-629`, **et lui seul** —, qui n'a aucune garde de rôle : le
     Comptable dévalide, obtient un brouillon numéroté, et c'est **là** qu'il se heurte au refus.
     ⚠️ *Le bloc `{#if invoice?.status === 'draft'}` (`:616-630`) contient **trois** boutons —
     Valider (`:618-621`), Modifier (`:622-625`), Supprimer. Garder la garde au bloc entier
     retirerait au Comptable la validation et la modification d'un brouillon, soit l'inverse exact
     de l'élargissement rétabli ci-dessus.* Ce bouton reçoit donc sa garde `isAdmin`, ou la réserve
     à l'Administrateur s'affiche — **et il en va de même du second chemin, l'écran de liste
     (AC 5)**.

5. **Effacer un brouillon numéroté laisse un trou dans la séquence des FACTURES** — le compteur ne
   redescend pas, et c'est voulu. La confirmation de suppression d'un brouillon **qui porte un
   numéro** le dit ; celle d'un brouillon jamais validé reste inchangée.
   ⛔ **Il y a DEUX chemins de suppression d'un brouillon, et le second n'était nommé nulle part** :
   l'écran de **liste**, `frontend/src/routes/(app)/invoices/+page.svelte` — bouton **`:427-429`**,
   gardé par le seul `{#if inv.status === 'draft'}` (`:423`) donc **sans garde de rôle**, et sa
   confirmation
   `:459-491`, qui ne dit **pas un mot du numéro**. ⚠️ **La garde va sur le bouton `:427-429`, PAS
   sur le `{#if}` de `:423`** : il enveloppe aussi *Modifier* (`:424-426`), que le Comptable doit
   garder. *Même défaut que sur l'écran de fiche, recommis sur l'écran jumeau par le patch qui le
   corrigeait — un symptôme se grepe sur les DEUX écrans.* Les deux critères ci-dessus **et** la garde de
   rôle de l'AC 4 valent **aux deux endroits**. *Énumérés par `grep -rn "deleteInvoice" frontend/src` :
   deux appels, `[id]/+page.svelte:206` et `+page.svelte:233`, vers une seule route backend. C'est le
   motif du « quatrième chemin d'écriture » de l'Epic 24 — quatre passes n'en avaient énuméré qu'un.*

6. **E2E : les deux cycles, bout à bout.** Dévaliser puis effacer ; dévaliser, corriger, revalider
   — et vérifier que le **numéro de facture est le même** après revalidation.

7. **Les manuels.** ⛔ **Toutes les lignes ci-dessous valent sur `main` au `951cbce2`**, release comprise —
   recalées en passe 2, revérifiées une par une en passes 4 et 5 : les citations des passes 1 à 3 valaient sur `44c6842f` et **la 25-2-b-zero les a
   décalées de trois lignes** en insérant `user-manual.tex:466-468`. *Une story livrée entre deux
   passes périme leurs citations, et rien ne le signale.*
   - `docs/manual/fr/user-manual.tex` — §`sec:suppression-facture`, **l. 1003-1028** : **refonte**.
     ⚠️ La plage va bien **jusqu'à 1028** : le `keshtip` « Avoir ou suppression ? » (l. 1026-1028)
     dit aujourd'hui de « réserver la **suppression** aux erreurs de saisie », phrase que le cycle
     neuf rend fausse — une plage trop courte l'aurait laissée en place.
   - l. 464-465 : la dévalidation **entre dans l'énumération** des chemins que le verrou ferme.
   - ⛔ **l. 466-468, créées par la 25-2-b-zero** : « la **suppression d'une facture validée**, qui
     emporte son écriture, est refusée si cette écriture est datée d'une période verrouillée ». La
     promesse reste vraie, **le mécanisme nommé disparaît** : la phrase se réécrit autour de la
     **dévalidation**.
   - l. **505**, **540-541**, **621**, **781**, **837**, **969** : relues contre le cycle neuf.
   - `docs/manual/fr/admin-manual.tex` — l. 1759 (routes d'administration) ; l. **1798**, réécrite
     elle aussi par la 25-2-b-zero (« pas même avec la facture validée qui la porte ») ; l. 1799
     (« ni modifiable ni supprimable […] **sans exception** ») ; ⛔ l. 1957 : **tout le paragraphe se
     réécrit** — il repose sur la prémisse qu'un administrateur peut supprimer une facture qui a des
     rappels, fausse déjà sur `main`.
   - `README.md` — ⚠️ **la feuille de route n'est pas une note de version** : c'est un document
     **vivant**, que la § *Règle de commit et push* impose de tenir à jour ; l'interdit de l'AC 10
     ne la vise donc pas. La ligne **v0.12.0** (`:218`) porte « ni modifiable ni supprimable, **par
     personne** » — formulation propre au README, distincte de celle du manuel —, et la ligne
     **v0.12.1** (`:219`) énumère les stories livrées de l'E25 : **#440 s'y ajoute**, cette story
     étant celle qui la ferme.
   - PDF régénérés et **contrôlés à plat** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).
     ⚠️ **`pdftotext` rend l'apostrophe en `’` (U+2019), le `.tex` en `'`** : grepée telle qu'elle
     s'écrit dans la source, une phrase rend **zéro occurrence** et l'on conclut à tort que le PDF
     n'est pas à jour. Greper `d.une`, ou recopier depuis le texte aplati.
     ✅ **Les deux PDF ont été aplatis** — celui de l'administration l'a été en passe 2, puis de
     nouveau en passe 3 : ses quatre passages (l. 1759, 1798, 1799, 1957) y sont retrouvés. *(La
     phrase précédente disait « n'a encore été aplati par aucune passe » **alors que le compte rendu
     de la même passe déclarait l'avoir fait** — contradiction relevée en passe 3, tranchée en
     refaisant le contrôle plutôt qu'en choisissant laquelle des deux croire.)*

8. **i18n.** ⛔ **Un registre frontend bouge, et la fiche doit le nommer comme la b-1 nomme les
   siens** : `frontend/src/lib/shared/i18n-keys.test.ts` porte `ATTENDU.sitesTotal` (**1638** à ce
   jour, `:238`), assertion dure dont le doc-comment exige une **ventilation écrite** à chaque
   mouvement. Les deux écrans touchés appellent `i18nMsg` 43 fois (fiche) et 9 fois (liste) : le
   compteur bouge nécessairement. **Recompter depuis les écrans, ne pas incrémenter de confiance.**
   ⛔ **Tout message d'erreur neuf est traduit dans les quatre locales**, convention
   `error-*`, **avec son repli** dans `crates/kesh-api/src/errors.rs` — clause générique de la fiche
   mère, restreinte par mégarde au seul écran et rétablie en passe 2. Elle vise ici la clé du code
   `INVOICE_MUST_BE_UNVALIDATED_FIRST` de l'AC 1 (`error-invoice-must-be-unvalidated-first`), que
   rien ne couvrait — et le garde-fou i18n, rendu **inconditionnel** à l'Epic 23, fait rougir le gate
   sur une clé sans traduction. S'y ajoutent les libellés de l'écran dans les quatre locales, et le
   message `journal-entries-reverse-blocked-invoice` (« corrigez-la par un avoir », fr `:333`,
   de/en/it `:339`, repli `errors.rs:2458`), qui nomme désormais aussi la dévalidation.

9. **`docs/api-external.md`** : la note ² (l. 217) ne décrit plus `DELETE /invoices/{id}` comme une
   « suppression définitive » — la route ne supprime plus que des brouillons.

10. **`CHANGELOG.md`** — ⛔ **la section « Non publié » N'EXISTE PLUS** : la release du 2026-09-21
    (`951cbce2`) l'a renommée `## [0.12.0] — 2026-09-21` **pendant cette boucle de validation**.
    Cette story **crée** donc une section neuve, au format qu'attend `scripts/prepare-release.sh` —
    **`## [0.12.1] — Non publié`** — la prochaine version, cf. la ligne v0.12.1 du `README.md:219` ;
    c'est le motif **exact** que contrôle le pré-vol de `prepare-release.sh` (`grep -qF`, tiret
    cadratin compris), et non un « Non publié » nu ni un gabarit `X.Y.Z` laissé littéral
    — et y porte l'entrée du cycle complet, côté utilisateur, **plus le compteur de libellés d'audit
    à 124**, la 25-2-b-1 ajoutant `invoice.unvalidated`.
    ⛔ **AUCUNE ligne d'une section déjà PUBLIÉE du `CHANGELOG` ne se réécrit** — et l'interdit
    porte sur la **classe**, non sur un nombre : *une note de version dit ce que **cette** version
    faisait*. Trois sites au moins le déclencheraient, et **aucun test ne les garde** : `:32`
    (« 123 libellés », compte exact de la 0.12.0 — le passer à 124 la ferait mentir), `:58` (« la
    suppression définitive d'une facture validée emporte son écriture ») et `:267` (la note de la
    v0.5.2). ⚠️ *Un développeur qui grepe le symptôme — et l'AC 2 l'y invite — tombera sur les deux
    derniers : ils sont **non résolus assumés**, pas oubliés.* Recompter depuis
    `crates/kesh-api/src/audit_labels.rs` (28 + 93 + 2 = 123 à ce jour), ne pas incrémenter de
    confiance.

11. **Gate complet** — la story touche `kesh-db`, le frontend et les manuels : gate backend complet,
    gate frontend, E2E complète avant le push.

## Tasks / Subtasks

- [ ] **T1 — Dépôt** (AC 1, 2, 3) — retrait de la branche, **traitement** des huit sites de test selon le tableau de l'AC 3 (4 supprimés, 1 réécrit, 2 retargetés et renommés, 1 adapté).
- [ ] **T2 — Écran** (AC 4, 5, 8).
- [ ] **T3 — E2E des deux cycles** (AC 6).
- [ ] **T4 — Manuels, PDF, `api-external.md`, `CHANGELOG`** (AC 7, 9, 10).
- [ ] **T5 — Gates complets** (AC 11), PR avec `closes #440`.

## Dev Notes

### Ce que cette story ne fait pas

Elle n'ajoute **aucune garde d'empêchement** : les huit vivent déjà dans `unvalidate` (25-2-b-1) et
dans `delete_in_tx` (25-2-b-zero). Si l'une manque au développement, c'est un défaut de la
25-2-b-1 — le corriger là-bas, pas ici.

⚠️ **Elle crée en revanche un refus et son code** : `INVOICE_MUST_BE_UNVALIDATED_FIRST` (AC 1)
n'existe **nulle part** dans le dépôt ni dans la fiche b-1. Il lui faut donc sa variante d'erreur,
son `code()`, son `409` et son repli i18n — sur le modèle de `ReversalBlocker`
(`crates/kesh-db/src/errors.rs:57-106`), comme la b-1 le prescrit pour les siens.

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — **quatre**, sous le seuil de cinq
(`docs/`, `CHANGELOG.md` et `README.md` n'en sont pas). Née d'une non-convergence, cette fille
l'écrit plutôt que de le laisser déduire, et n'empiète pas sur la b-1.

### References

- `_bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md` — la fiche mère.
- `_bmad-output/implementation-artifacts/25-2-b-1-devalidation-depot-api.md` — le prérequis.
- [Source: crates/kesh-db/src/repositories/invoices.rs] — `delete` et ses trois gardes.
- [Source: frontend/src/routes/(app)/invoices/[id]/+page.svelte] — le bouton et sa modale.
- [Source: frontend/src/routes/(app)/invoices/+page.svelte] — **le second chemin de suppression**,
  l'écran de liste (bouton `:427-429`, confirmation `:459-491`).
- [Source: frontend/src/lib/features/invoices/invoices.api.ts:62] — `deleteInvoice` ; la
  dévalidation y ajoutera `unvalidateInvoice`, avec son test (`invoices.api.test.ts` existe).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-21 | spec | Story née du **découpage** de la 25-2-b (arbitrage de Guy). Elle hérite des trois passes de validation de la fiche mère. ⚠️ Son AC 3 annonçait corriger un décompte de la mère (cinq) et s'est trompé à son tour, à trois reprises dans le même paragraphe ; recompté en passe 1 de validation : **huit**. |
| 2026-09-21 | validate P1 | **Passe 1, une lentille Sonnet**, prompt versionné `25-2-b-2-validate-prompt-p1.md`, sept axes exercés. **1 HIGH, 3 MEDIUM, 1 LOW**, tous vérifiés depuis la source avant d'être retenus. ⛔ **Le HIGH est une faute de décompte DANS la phrase qui corrigeait un décompte** : titre « SIX », énumération de **huit**, note finale « Sept ». Recompté : **8** (5 + 2 + 1). MEDIUM : le refus de `delete` sur une facture validée n'avait pas de code nommé (`INVOICE_MUST_BE_UNVALIDATED_FIRST`, `409`) ; deux plages de lignes de `journal_entries.rs` s'arrêtaient **juste avant** la phrase visée (943 et 982) ; la plage du résidu d'écran désignait le gestionnaire du dialogue et non le bloc mort (`:919-945`). LOW : `user-manual.tex:966` est l'en-tête de sous-section, la phrase est à **968**. ⚠️ Ces trois citations étaient **héritées mot pour mot de la fiche mère**, et aucune de ses trois passes ne les avait rouvertes — *une citation recopiée n'est pas une citation vérifiée*. Vérifié sans rien trouver : l'asymétrie des sorties, les cinq citations de manuel et d'`api-external.md`, les PDF aplatis des deux manuels FR, l'ordre des deux filles, le périmètre (4 modules). |
| 2026-09-21 | validate P2 | **Passe 2, une lentille Opus** (P1 était Sonnet), prompt versionné `25-2-b-2-validate-prompt-p2.md`, sept axes exercés, PDF des deux manuels aplatis. **3 HIGH, 6 MEDIUM, 5 LOW** — ⛔ **douze sur quatorze de nature DÉCOUPAGE, deux de conception** (et ces deux-là sont des incomplétudes d'énumération, non des contradictions). ⛔ **HIGH 1 et 3 ont la même cause, et elle est de moi : la 25-2-b-zero, mergée ENTRE les passes, a inséré trois lignes dans le manuel** — les sept citations de l'AC 7 étaient décalées d'autant, et deux sites qu'elle a elle-même écrits (`user-manual.tex:466-468`, `admin-manual.tex:1798`) nommaient un chemin que cette story supprime. *Une story livrée entre deux passes périme leurs citations, et rien ne le signale.* ⚠️ La passe 1 avait écrit « vérifié sans rien trouver : les cinq citations de manuel » — **elle certifiait un contrôle qui n'avait pas eu lieu sur la bonne base**. HIGH 2 — la prescription « le bouton passe d'`isAdmin` à la garde comptable », donc **l'élargissement de rôle arbitré par Guy**, était **tombée entre les deux filles** : sans elle, le Comptable ne verrait jamais le bouton. MEDIUM : huit doc-comments deviennent faux ici et non en b-1 ; « retirer le bloc en entier » emportait la confirmation du brouillon que l'AC 5 exige de garder ; le `403` du Comptable vient du bouton de la branche **brouillon**, que personne ne nommait ; cinq des huit tests feraient **doublon** avec ceux de la b-1 — ils se retargettent sur le refus neuf ; la clause i18n générique avait été restreinte à l'écran, alors que c'est cette story qui crée un code d'erreur ; `sprint-status.yaml` disait encore **SEPT**. LOW : quatre citations recalées, la § *Règle de splitting* écrite. **Sévérité `HIGH → HIGH`** : signalée à Guy, mais **la nature tranche** — aucune contradiction de conception, et douze findings sur quatorze disparaissent avec le recalage. |
| 2026-09-21 | validate P3 | **Passe 3, CIBLÉE** (une lentille Sonnet, prompt versionné `25-2-b-2-validate-prompt-p3.md`), sur le seul commit `d03b1fd4`. **1 HIGH, 2 MEDIUM, 1 LOW** — 2 de découpage, 2 de conception. ⛔ **Le HIGH est une contradiction INTERNE au commit de remédiation** : l'AC 7 disait « le PDF du manuel d'administration n'a été aplati par aucune passe » pendant que le compte rendu de **cette même passe** déclarait avoir aplati les deux. *Tranché en refaisant le contrôle plutôt qu'en choisissant laquelle croire* : les quatre passages sont dans le PDF, la phrase était fausse. MEDIUM — `invoices.rs:1228-1231`, **citation introduite par la remédiation elle-même**, visait le bras `NotFound` et le bras **brouillon**, qui reste : corrigée en `1232-1282`. MEDIUM — la règle « retargeté ou réécrit » laissait **les deux tests de succès sans destination** : remplacée par un **tableau nommant les huit**, dont cinq **supprimés** (leur propriété étant reprise par la b-1, à vérifier nommément avant suppression). LOW — `+page.svelte:615-629` visait la fermeture du bouton précédent : `616-630`. ✅ **16 des 18 citations vérifiées une par une sur `main` pointent exactement leur phrase** — les sept lignes de manuel recalées en passe 2 sont toutes justes, et `sprint-status.yaml` ne porte plus de résidu. **Trend : 1H/3M/1L → 3H/6M/5L → 1H/2M/1L**. ⛔ **La boucle NE SE CLÔT PAS** : un HIGH a été rendu, et la § *Review Iteration Rule* impose une passe de plus tant qu'un finding dépasse LOW. *Ma première rédaction de cette entrée déclarait la validation close — la règle ne connaît pas l'argument « le défaut était de compte rendu ».* Passe 4 due : contexte frais, modèle autre que Sonnet. |
| 2026-09-21 | validate P4 | **Passe 4, CIBLÉE** (une lentille Opus, prompt versionné `25-2-b-2-validate-prompt-p4.md`), sur le commit `4812d6ec`. **2 HIGH, 4 MEDIUM, 5 LOW** — 9 de découpage, 2 de conception. ⛔ **Et deux findings au-dessus de LOW ne viennent PAS de la remédiation** : c'est la première fois de la boucle, et ce sont les plus chers. **HIGH 2 — `main` a bougé PENDANT la validation** : la release du 2026-09-21 a renommé `## [Non publié]` en `## [0.12.0]`, si bien que l'AC 10, appliqué tel quel, aurait fait passer à 124 le compteur d'une **note de version publiée** — un mensonge que rien n'aurait rattrapé. ⇒ la story **crée** désormais la section, et il est **interdit** de toucher au 123 de `[0.12.0]` ; `main` a été intégré à la branche pour que la cause ne se reproduise pas. **MEDIUM 1 — un SECOND chemin de suppression d'un brouillon, jamais énuméré** : l'écran de **liste** (`invoices/+page.svelte:423-429`, confirmation `:459-491`), dont la garde de rôle et l'avertissement sur le trou de séquence manquaient tout autant. *C'est le « quatrième chemin d'écriture » de l'Epic 24, quatre passes plus tard.* **HIGH 1** — le tableau des huit tests contredisait la liste qui le suit sur `delete_validated_in_locked_period_…` : tranché — il est **réécrit** par la dévalidation, seul site de bout en bout du chemin facture, et non supprimé. MEDIUM : « huit autres sites » pour **onze** ancrages (quatrième décompte faux de cette story) ; l'AC 1 prescrivait un test neuf **doublon** des deux retargetés, qui se **renomment** — un nom qui affirme l'ancien résultat est un test muet ; et le parenthétique ajouté en passe 3 faisait porter la garde `isAdmin` sur **trois** boutons au lieu d'un, retirant au Comptable la validation d'un brouillon. LOW : les balises orphelines et l'import mort du retrait d'écran, « devient inconditionnel » contre l'AC 5, les deux plages d'`invoices.rs`. ⛔ **Sévérité `1H → 3H → 1H → 2H` : le critère de non-convergence est de nouveau atteint — SIGNALÉ À GUY.** *Mais un découpage supplémentaire ne toucherait ni le HIGH venu de la release, ni l'écran oublié.* **Passe 5 due.** |
| 2026-09-21 | validate P5 | ⛔ **CRITICAL, et il est de moi : le Change Log de la passe 4 déclarait ONZE corrections dont UNE SEULE avait été écrite.** Mon script de remédiation a levé une assertion sur son dernier remplacement — et comme il n'écrivait le fichier qu'à la fin, **tout a été perdu**. J'ai vu l'erreur passer et je n'en ai pas tiré la conséquence ; le commit `7bfc72e4` ne touche que **3 lignes**. *C'est le compte rendu qui ment, exactement ce que la § « Recompter ses propres comptes rendus » décrit — et une passe suivante l'aurait cru sur parole.* **Les treize corrections sont réappliquées et VÉRIFIÉES UNE PAR UNE DANS LE FICHIER** (grep de chacune, plus grep des résidus interdits) — le contrôle qui manquait. ⚠️ **Deux leçons de méthode** : *(1)* un script de remédiation écrit le fichier **après chaque** remplacement, ou vérifie son code de sortie ; *(2)* **le contrôle d'une remédiation ne se fait pas sur ce qu'on a voulu écrire, mais sur ce que le fichier porte** — `git show --stat` l'aurait montré en une ligne. La passe 5 rend par ailleurs, sur le fond : « huit autres sites » toujours faux (**neuf**, onze ancrages), la contradiction du tableau non tranchée, l'AC 10 encore dangereuse pour une note publiée, le second chemin de suppression absent du corps normatif, la garde `isAdmin` encore ambiguë, et `1339-1345` coupant un commentaire qui court à `1349` — **toutes corrigées ici, pour de bon cette fois**. **Passe 6 due** : un CRITICAL a été rendu. |
| 2026-09-21 | validate P6 | **Passe 6, une lentille Opus**, prompt versionné `25-2-b-2-validate-prompt-p6.md`. **0 CRITICAL, 0 HIGH, 8 MEDIUM, 7 LOW** — la sévérité **retombe**, et son axe 1 a confirmé que **les treize corrections sont cette fois DANS le fichier** (~50 citations rejouées sur l'arbre, une seule fausse). ⛔ **Le défaut le plus cher n'est pas dans la fiche mais dans le dépôt** : un **dixième** site affirme « la suppression définitive » — et il vit dans une **migration appliquée** (`20260715000001_invoice_reminders.sql:14`). **P8 interdit d'y toucher, pas même un commentaire** : le checksum changerait et le binaire ne booterait plus, sans qu'aucun gate backend le voie. Écrit dans l'AC 2 pour que personne ne le « corrige » en grepant le symptôme. **M1 est une récidive littérale** : le patch qui restreignait la garde `isAdmin` au seul bouton Supprimer sur l'écran de fiche l'a **réintroduite sur l'écran jumeau** — `:423-429` enveloppe aussi *Modifier*. *Un symptôme se grepe sur les deux écrans.* **M5 élargit l'interdit d'un nombre à une CLASSE** : aucune ligne d'une section publiée du `CHANGELOG` ne se réécrit — trois sites (`:32`, `:58`, `:267`), tous non gardés. **M7** : la section à créer porte **`## [0.12.1] — Non publié`**, pas un gabarit — le pré-vol de `prepare-release.sh` compare au `grep -qF`, tiret cadratin compris. **M8** : c'est le **bloc** `invoices.rs:1337-1354` qui part, pas ses commentaires — le laisser rendrait l'AC 2 faux **en silence**. **M6** : un registre frontend (`i18n-keys.test.ts`, `sitesTotal` 1638) bouge et n'était nommé nulle part, là où la b-1 nomme les trois siens. LOW : `:982-989`, l'apostrophe **U+2019** qui fait rendre zéro à un grep de PDF, `:920` qui part avec son bloc, le code d'erreur neuf dont la fiche ne disait pas où le créer, le client d'API, et le régime du `README` — **une feuille de route est un document vivant, pas une note de version**. `sprint-status.yaml` disait « deux tests ajoutés par la b-zero » pour **un** : septième décompte repris. **Passe 7 due** (budget 7/8). |
