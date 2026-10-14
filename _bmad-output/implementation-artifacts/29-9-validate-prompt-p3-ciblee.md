# Prompt — validation P3 **ciblée** de la spec, Story 29-9 (trois petits correctifs : #649, #655, #646)

*Versionné le 2026-10-14. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P2** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Tendance des MEDIUM distincts : P1 6 → P2 2 (R1 = F1, issu de la
remédiation P1 ; F2, de la conception d'origine) ; 0 HIGH, 0 CRITICAL aux deux passes.*

Dépôt `/home/gcorbaz/devel/kesh-29-9`, branche `story/29-9`. Le code au sol est celui de `fcd2ae75` : la fiche est une
spec, rien n'est encore implémenté ; les dépendances front sont dans `frontend/node_modules` (`@sveltejs/kit` 2.55.0,
`svelte` 5.55.1).
**Périmètre : `git show 13ac02c5`** (remédiation P2), soit un **diff unique, aplati** — ne lis pas la séquence des
commits antérieurs. Fichiers touchés : la fiche `_bmad-output/implementation-artifacts/29-9-trois-petits-correctifs.md`,
le registre `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-9.md` (C-29-9-1, 8, 11, 13, 16 annotés ;
C-29-9-17 et 18 ajoutés), `sprint-status.yaml` (ligne `29-9-…`). Rapports P2 que ce commit remédie :
`/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p2-{R,F}.md`. Change Log de la fiche, entrée « Validation P2
remédiée ». Fiches voisines, **en lecture seule**, lues telles qu'elles sont :
`/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`,
`/home/gcorbaz/devel/kesh-29-6/_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md`,
`/home/gcorbaz/devel/kesh-29-8/_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md`.

⚠️ **Toute vérification de présence ou d'absence se fait par `grep -nF "<chaîne>" <fichier>` sur le fichier FINAL**
(l'arbre de travail, égal à `13ac02c5` pour ces trois fichiers), jamais en lisant les numéros de ligne du diff : un
hunk du diff ne donne pas la ligne du fichier final. Un finding qui affirme qu'un correctif « manque » sans sortie
`grep -nF` copiée ne compte pas.

## La lentille — chasseur de régressions de la remédiation P2

Pour **chaque hunk** du commit, vérifie contre le code au sol :

1. **Commentaire de module `kesh-db` réécrit (AC4, C-29-9-16 annoté et renversé — décision de l'orchestrateur)** :
   le texte au sol (`sed -n 18,19p crates/kesh-db/src/repositories/reconciliation.rs`) est-il bien celui que la fiche
   cite ? La phrase de remplacement prescrite est-elle **vraie** au code de la route
   (`crates/kesh-api/src/routes/reconciliation.rs:598-602`, `:621-628`) — défaut 100, plafond 500, `limit + 1` ? La
   fiche dit-elle juste sur l'exception *kesh-db* : **gate complet au seul commit qui touche ce fichier** (T1) et à un
   patch de revue qui le retoucherait, **pas** à toute la boucle (citer la phrase de `CLAUDE.md` § *Pendant une boucle
   de revue*) ? Le décompte de modules (§ *Dérogation*, C-29-9-1 annoté) est-il juste au regard du seuil « plus de 5 » ?
   Reste-t-il, dans la fiche ou le registre, une phrase **vivante** qui dit encore le commentaire « vrai » ou
   « non réécrit » (`grep -nE "reste vrai|non réécrite|laissé tel quel"` ; une phrase d'origine suivie de son
   annotation datée n'est pas vivante) ?
2. **Garde d'`onUpdated` (AC6, C-29-9-17, V17, M19 — décision de l'orchestrateur)** : le scénario est-il vrai au code
   (`frontend/src/routes/(app)/bank-import/profiles/[id]/+page.svelte:33-35`, `:48` ;
   `frontend/src/lib/features/bank-import/BankProfileForm.svelte:44-60`, `:113-116`) — en particulier
   `existing.id` lu **avant** l'`await` ? La fiche affirme qu'une garde par fermeture (`onSuccess={onUpdatedFor(seq)}`)
   ne tient pas parce que Svelte 5 lit les props en direct (`$$props.onSuccess`) : cette affirmation est-elle déclarée
   pour ce qu'elle est (compilation d'essai hors dépôt), et V17 suffit-il à la trancher ? La garde retenue
   (`updated.id !== Number(rawId)`) a-t-elle un cas où elle laisse passer l'écriture fautive, ou en refuse une juste ?
   V17 est-il **rouge au sol** (le formulaire de 5 n'y est même pas démonté) et **tue-t-il M19** (second envoi avec 5
   au lieu de 6) ? La fabrique de mock (`profile-page.test.ts:12-15`) gagne-t-elle bien ce que le formulaire importe
   (`BankProfileForm.svelte:11-14`) ? L'écart entre la lettre de la décision (« même garde de séquence ») et la forme
   retenue est-il écrit, et nulle part caché ?
3. **Bouton désactivé pendant un lot (AC1, V18, M21)** : au code de
   `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `busy` passe-t-il à `true` (`:194`,
   `:239`) sans que `loading` change avant le `load()` final, si bien que le bouton reste affiché pendant le lot ?
   V18 est-il réalisable avec les patrons du fichier de test (cocher une ligne, promesse différée), rouge au sol, et
   tue-t-il M21 ?
4. **Mutation M20 et la parenthèse de l'AC4 (R2 = F3)** : calcule toi-même, sur la ligne réelle
   (`routes/reconciliation.rs:625`, `:628`), que `limit` pour `limit + 1` laisse vert le test à exactement 500 et
   rougit le test à 501 et `get_proposals_paginates_at_100_default` (150 transactions, défaut 100). M18 cite-t-il
   désormais `:628` ? Reste-t-il `:627` vivant ?
5. **Étape 0 de l'AC9 en delta, repli `reconciliation.limits.ts` (R4, R6, F7)** : la prescription n'impose-t-elle
   plus nulle part une fabrique **complète** qui effacerait les entrées d'une fiche voisine ? Les faits cités sont-ils
   exacts : `ReconciliationProposals.test.ts:22-27`, `:44-49`, `:53` ; `frontend/src/lib/app/stores/auth.svelte.ts:34-36`
   (`BroadcastChannel`) ; `frontend/src/lib/shared/utils/api-client.ts:14-20` (imports) ;
   `frontend/src/routes/(app)/bank-import/bank-import-page.test.ts:19-20` ? Le repli exige-t-il bien l'import
   **direct** par le composant ? « V2-V5 rougissent ; V1 et V12 verts au sol » est-il cohérent avec T2 ?
6. **Formulation `page_changed` (F5)** : au source (`frontend/node_modules/@sveltejs/kit/src/runtime/client/client.js`,
   `:707`, `:763-772`, `:1888`), la formulation retenue est-elle exacte (URL, erreur, formulaire, données) ? Reste-t-il
   un « à chaque navigation » / « à toute navigation » **vivant** qui ne soit ni dans une entrée datée annotée, ni
   dans le Change Log de la P1 (`grep -nE "chaque navigation|toute navigation"` sur la fiche et le registre) ?
7. **Zones partagées (R6, F8, C-29-9-11 annoté)** : les numéros de ligne cités des fiches voisines (29-8 `:370`,
   `:387-388`, `:474`, `:550` ; 29-6 `:636`, `:551` ; 29-3 `:532`, `:549`) désignent-ils ce que la fiche dit, dans les
   fiches telles qu'elles sont aujourd'hui ? La règle « avis et bouton immédiatement après `</table>`, avant toute
   modale » est-elle écrite de façon cohérente à l'AC1, au tableau et au registre ?
8. **`untrack` écarté (F6, C-29-9-18)** : la raison (garde intuable) est-elle cohérente avec C-29-9-14, et les sites
   cités (`frontend/src/lib/features/bank-import/bank-profile.api.ts:19-21`, `api-client.ts:181-199`, `:267-280`,
   `:329-336`, `:392-399`) sont-ils exacts ?
9. **Manuel et propagation (R9)** — axe obligatoire : la liste des occurrences légitimes attendues **après** la story
   (§ *Propagation*) est-elle vraie au regard de ce que l'AC8 prescrit pour `docs/manual/fr/user-manual.tex` (`:2045`,
   `:2057`, `:869-874`) et pour `reconciliation-unreject-success` ×4 ? Contrôle le **PDF** au sol, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "100 plus récentes.\{0,80\}"` —
   les deux phrases que l'AC8 réécrit y sont-elles, et la liste « ce qui ne doit plus sortir » les vise-t-elle ?
10. **Recomptes et historique** : V1-V18 (18), M1-M21 (21), C-29-9-1 à 18 (18), 10 AC, T0-T9 — recompte à la source
    (`grep -oE "^\s+- V[0-9]+"`, `grep -cE "^\| M[0-9]+"`, `grep -c "^### C-29-9-"`). Le bilan de l'entrée « Validation
    P2 remédiée » (18 findings, quatre paires, 14 distincts, 2 MEDIUM, 12 LOW) est-il juste contre les titres des deux
    rapports (`grep -nE "^### (R|F)[0-9]+"`) ? Le commit a-t-il **réécrit** une entrée datée (Change Log P1, annotation
    P1 du registre) au lieu de l'annoter ? La ligne `29-9-…` de `sprint-status.yaml` dit-elle « 1 à 18 » ?

## Ce que tu rends

Findings numérotés `P3-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne` du fichier final,
**preuve** (commande et sortie copiée, `grep -nF` pour toute présence ou absence), origine (remédiation P2 ou
conception d'origine), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (les dix ci-dessus, un par
un, avec ce qui a été fait ou pourquoi pas) — un « 0 finding » sans elle ne compte pas comme passe, et un axe déclaré
exercé sans commande copiée compte comme non exercé.
Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p3-ciblee.md` ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p3-ciblee.md`. Aucune commande qui
écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`restore`/`stash`/
`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, sous **aucune** forme et dans **aucun** worktree, `cargo`,
`npm`, `npx`, `node`, `docker`, `mysql`/`mariadb` et toute requête SQL, `gh` en écriture, `sed -i`, et **tout
interpréteur** (`python`, `python3`, `perl`, `ruby`, `node`, `deno`, `bash`/`sh` sur un script ou avec `-c`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `git log`/`show`/`diff`/`blame`, `gh api` et
`gh issue view` en lecture, `pdftotext` vers la sortie standard.
