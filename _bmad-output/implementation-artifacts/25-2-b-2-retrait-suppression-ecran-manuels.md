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
   message **oriente vers la dévalidation**, et non sous le générique `ILLEGAL_STATE_TRANSITION` ;
   un test neuf l'exerce : la branche retirée ne doit pas redevenir
   un chemin silencieux.

2. **`enforce_immutability = false` n'a plus qu'un appelant de production** — `unvalidate`. Le
   drapeau **reste dans `delete_in_tx`**, jamais chez l'appelant : une garde posée chez l'appelant
   laisserait la fonction nue pour le suivant. ⚠️ **Les trois doc-comments qui nomment
   `invoices::delete` comme seul appelant** (`journal_entries.rs:943`, `:982` ;
   `invoices.rs:1339-1340`) sont réécrits **par la 25-2-b-1**, qui les périme en créant le second
   appelant. ⛔ **Mais huit autres sites affirment encore que `invoices::delete` détruit l'écriture
   d'une facture validée**, et ils deviennent faux **ici** — relevé en passe 2, aucun test ne les
   garde :
   - `journal_entries.rs:943` et `:982`/`:988` — **deuxième réécriture** : b-1 les fait dire « deux
     appelants », b-2 en retire `invoices::delete` **et** sa justification (« sous ses trois gardes
     propres »), qui déménage chez `unvalidate` ;
   - `journal_entries.rs:970` — « (ex. `invoices::delete` d'une facture validée — #219) » ;
   - `crates/kesh-api/src/lib.rs:234` et `:416` ;
   - `crates/kesh-api/src/routes/invoices.rs:5` (doc de module) ;
   - `crates/kesh-api/tests/invoice_delete_e2e.rs:1` et `:6` (doc de module) ;
   - `frontend/src/routes/(app)/invoices/[id]/+page.svelte:68`, `:95`, `:920`.
   Ceux d'`invoices.rs:**1232-1282**` (l'arme `Some(inv) if inv.status == "validated"` en entier),
   `:1331` et `:1339-1345` **partent avec la branche** : ils ne se réécrivent pas. ⚠️ *La plage
   citée en passe 2 — `1228-1231` — visait le bras `NotFound` et le bras **brouillon**, qui reste :
   une citation introduite par une remédiation, et fausse. Relevée en passe 3.*

3. **HUIT tests existants changent d'objet, et PAS tous de la même façon** — relevé en passe 2 :
   cinq d'entre eux (motifs 1, 2, 3, 5, 6) réaffirmeraient mot pour mot ce que la **25-2-b-1** teste
   déjà sur `unvalidate`. La consigne uniforme « réécrire contre `unvalidate` » produirait des
   doublons ; ⛔ **ce qui n'est couvert nulle part, c'est le refus neuf** de `delete` sur une facture
   validée (`INVOICE_MUST_BE_UNVALIDATED_FIRST`), pour lequel l'AC 1 ne prévoit qu'un test. Chaque
   test est donc traité **nommément**, et non par une règle à appliquer au jugé — relevé en passe 3,
   où la règle en bloc laissait les deux tests de **succès** sans destination :

   | Test | Ce qu'il devient | Pourquoi |
   |---|---|---|
   | `test_delete_validated_unpaid_open_fy_removes_invoice_and_je` | **retargeté** : `delete` sur une facture validée → `INVOICE_MUST_BE_UNVALIDATED_FIRST` | c'est le test du **succès** qui disparaît : après l'AC 1, aucun succès n'existe plus sur ce chemin |
   | `delete_validated_as_admin_returns_204` | **retargeté** : même refus, par l'API (`409`) | idem, côté HTTP |
   | `…_paid_is_rejected`, `…_credited_by_avoir_…`, `…_with_reminders_…`, `…_in_closed_fy_…`, `delete_validated_in_locked_period_…` | **supprimés ici**, leur propriété étant reprise par les tests d'empêchement de la **25-2-b-1** (motifs 1, 2, 3, 5, 6) | réécrits contre `unvalidate`, ils **répéteraient mot pour mot** ceux de b-1 ; ⛔ **ne les supprimer qu'après avoir vérifié nommément que le test de b-1 existe** pour chaque motif |
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
     le contenu de la branche `{:else}` (`:944`, la confirmation du brouillon), qui devient
     inconditionnel — l'AC 5 l'exige et l'enrichit. Deviennent morts aussi
     `let deleteConfirmText = $state('')` (`:97`, et son commentaire `:95`), sa remise à zéro dans
     `onOpenChange` (`:911`) et la condition `:956-958`.
   - ⛔ **La garde de `:729` change de rôle, et c'est un ÉLARGISSEMENT** (arbitrage de Guy du
     2026-09-19, perdu entre les deux filles et rétabli en passe 2) : `isAdmin && !invoice.paidAt`
     devient la **garde comptable** (`canManage`, `:65-67`) — jusqu'ici, seul l'Administrateur
     pouvait faire disparaître l'écriture d'une facture. Laissée telle quelle, **le Comptable ne
     verrait jamais le bouton « Dévalider »** et l'arbitrage ne serait pas livré.
   - ⚠️ **Et `!invoice.paidAt` sous-couvre le motif 1** : une facture *partiellement* réglée n'a pas
     de `paid_at`. L'écran lit le résiduel, ou assume le refus serveur — au choix, mais écrit.
   - ⚠️ **L'asymétrie des deux sorties s'affiche** (arbitrage de Guy, 2026-09-19) : dévaliser est
     ouvert au **Comptable**, mais **effacer** reste réservé à l'**Administrateur** (`admin_routes`,
     décision de #219, fermée aux clés API). ⛔ **Le `403` en question vient du bouton de la branche
     BROUILLON** (`+page.svelte:616-630`, le bloc `{#if invoice?.status === 'draft'}` en entier), qui n'a **aucune** garde de rôle — relevé en passe 2, et
     nommé nulle part ailleurs, pas même dans la fiche mère : le Comptable dévalide, obtient un
     brouillon numéroté, et c'est **là** qu'il se heurte au refus. Ce bouton reçoit donc sa garde
     `isAdmin`, ou la réserve à l'Administrateur s'affiche.

5. **Effacer un brouillon numéroté laisse un trou dans la séquence des FACTURES** — le compteur ne
   redescend pas, et c'est voulu. La confirmation de suppression d'un brouillon **qui porte un
   numéro** le dit ; celle d'un brouillon jamais validé reste inchangée.

6. **E2E : les deux cycles, bout à bout.** Dévaliser puis effacer ; dévaliser, corriger, revalider
   — et vérifier que le **numéro de facture est le même** après revalidation.

7. **Les manuels.** ⛔ **Toutes les lignes ci-dessous sont recalées sur `main` au `951cbce2`** — revérifiées une par une en passe 4, après le merge de la release —, en
   passe 2 : les citations des passes 1 à 3 valaient sur `44c6842f` et **la 25-2-b-zero les a
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
   - `README.md:218` — « ni modifiable ni supprimable, **par personne** » (formulation propre au
     README, distincte de celle du manuel), et la ligne **v0.12.1** (`:219`) de la feuille de route,
     qui énumère les stories livrées de l'E25 : **#440 s'y ajoute**, cette story étant celle qui la
     ferme.
   - PDF régénérés et **contrôlés à plat** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).
     ✅ **Les deux PDF ont été aplatis** — celui de l'administration l'a été en passe 2, puis de
     nouveau en passe 3 : ses quatre passages (l. 1759, 1798, 1799, 1957) y sont retrouvés. *(La
     phrase précédente disait « n'a encore été aplati par aucune passe » **alors que le compte rendu
     de la même passe déclarait l'avoir fait** — contradiction relevée en passe 3, tranchée en
     refaisant le contrôle plutôt qu'en choisissant laquelle des deux croire.)*

8. **i18n.** ⛔ **Tout message d'erreur neuf est traduit dans les quatre locales**, convention
   `error-*`, **avec son repli** dans `crates/kesh-api/src/errors.rs` — clause générique de la fiche
   mère, restreinte par mégarde au seul écran et rétablie en passe 2. Elle vise ici la clé du code
   `INVOICE_MUST_BE_UNVALIDATED_FIRST` de l'AC 1 (`error-invoice-must-be-unvalidated-first`), que
   rien ne couvrait — et le garde-fou i18n, rendu **inconditionnel** à l'Epic 23, fait rougir le gate
   sur une clé sans traduction. S'y ajoutent les libellés de l'écran dans les quatre locales, et le
   message `journal-entries-reverse-blocked-invoice` (« corrigez-la par un avoir », fr `:333`,
   de/en/it `:339`, repli `errors.rs:2458`), qui nomme désormais aussi la dévalidation.

9. **`docs/api-external.md`** : la note ² (l. 217) ne décrit plus `DELETE /invoices/{id}` comme une
   « suppression définitive » — la route ne supprime plus que des brouillons.

10. **`CHANGELOG.md`**, section « Non publié » : l'entrée du cycle complet, côté utilisateur, **et
    le compteur de libellés d'audit** — « 123 libellés » devient **124**, la 25-2-b-1 ayant ajouté
    `invoice.unvalidated`. ⚠️ Elle le rend faux et ne touche pas ce fichier ; c'est donc ici qu'il se
    corrige. Recompter depuis `crates/kesh-api/src/audit_labels.rs` (types d'entité + actions +
    types d'auteur), ne pas incrémenter de confiance.

11. **Gate complet** — la story touche `kesh-db`, le frontend et les manuels : gate backend complet,
    gate frontend, E2E complète avant le push.

## Tasks / Subtasks

- [ ] **T1 — Dépôt** (AC 1, 2, 3) — retrait de la branche, réécriture des **huit** sites de test.
- [ ] **T2 — Écran** (AC 4, 5, 8).
- [ ] **T3 — E2E des deux cycles** (AC 6).
- [ ] **T4 — Manuels, PDF, `api-external.md`, `CHANGELOG`** (AC 7, 9, 10).
- [ ] **T5 — Gates complets** (AC 11), PR avec `closes #440`.

## Dev Notes

### Ce que cette story ne fait pas

Elle n'ajoute aucune garde métier : toutes vivent déjà dans `unvalidate` (25-2-b-1) et dans
`delete_in_tx` (25-2-b-zero). Si une garde manque au moment du développement, c'est un défaut de la
25-2-b-1 — le corriger là-bas, pas ici.

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — **quatre**, sous le seuil de cinq
(`docs/`, `CHANGELOG.md` et `README.md` n'en sont pas). Née d'une non-convergence, cette fille
l'écrit plutôt que de le laisser déduire, et n'empiète pas sur la b-1.

### References

- `_bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md` — la fiche mère.
- `_bmad-output/implementation-artifacts/25-2-b-1-devalidation-depot-api.md` — le prérequis.
- [Source: crates/kesh-db/src/repositories/invoices.rs] — `delete` et ses trois gardes.
- [Source: frontend/src/routes/(app)/invoices/[id]/+page.svelte] — le bouton et sa modale.

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
