# Prompt — validation P3 **ciblée** de la spec, Story 28-2 (une erreur d'API ne s'affiche plus « [object Object] »)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P2** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation ; D6, rétrospective de l'Epic 25 : Haiku réservé aux passes
ciblées). Trend des MEDIUM distincts : P1 7 → P2 2, 0 HIGH.*

Dépôt `/home/gcorbaz/devel/kesh-28-2`, branche `story/28-2` (rebasée sur `5aad8b35`, kickoff de l'Epic 28 ; base de
code inchangée, = `6f971d76`, v0.13.2). **Périmètre : le commit `eb9c68c8`** (remédiation P2), lu en **diff unique**
— ne lis pas la séquence des commits antérieurs :

```sh
git show eb9c68c8 -- _bmad-output/implementation-artifacts/28-2-erreurs-object-object.md \
  _bmad-output/implementation-artifacts/sprint-status.yaml
# Registre : le fichier de branche a été versé dans celui de l'epic. Diff utile :
git diff eb9c68c8~1:_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-2.md \
         eb9c68c8:_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md
```

Dans ce second diff, la section **Story 28-0** (C-28-0-1 à 7) apparaît en ajout : elle vient de `main`, **hors
périmètre**. Ce qui est à relire : les annotations « Révisé / Retiré en validation P2 » et les choix neufs
**C-28-2-25 à C-28-2-29**. Rapports P2 que ce commit remédie (à lire) :
`/home/gcorbaz/devel/kesh-gate-logs/28-2-validate-p2-F.md` et `28-2-validate-p2-R.md`.

## La lentille — chasseur de régressions de la remédiation P2

Pour **chaque hunk** du commit, vérifie contre le code au sol (`frontend/src`, à l'état de `6f971d76`) :

1. **Classe C retirée (C-28-2-25, AC5)** : les trois sites `hooks.client.ts:27`, `lib/app/stores/auth.svelte.ts:301`,
   `lib/shared/utils/api-client.ts:162` n'enveloppent-ils vraiment que des `fetch` directs (`auth.svelte.ts:229`,
   `api-client.ts:131`) ou la sortie d'`hydrate()` (`try` de `auth.svelte.ts:228-315`, `localStorage` `:222-226`) ?
   Reste-t-il dans la fiche ou le registre une phrase qui fait encore **toucher** `hooks.client.ts` /
   `auth.svelte.ts`, passer la valeur brute, ou annonce « classe C = 0 » / « 9 lignes disparaissent » hors des
   entrées passées ? Refais : `grep -nE "C-28-2-19|valeur brute|0 à la fin|9 lignes|septi|hooks\.client"` sur la
   fiche et le registre, et classe chaque occurrence (entrée historique voulue / résidu).
2. **Tests de soumission des modales (C-28-2-26, AC9, M15, M26)** : le montage prescrit est-il faisable tel
   qu'écrit ? `AccountAutocomplete.svelte:384-388` (sélection au `mousedown`), le sélecteur
   `input[aria-autocomplete="list"]`, le filtre actif/postable (`AccountAutocomplete.test.ts:57-79`), les filtres
   de classe (`ManualMatchModal.svelte:84-89`, `TransactionSplitModal.svelte:71-73`), `balanceExact` (`:108-113`),
   les `data-testid` cités (`manual-match-error`, `manual-match-bank-not-configured`, `split-error`,
   `split-bank-not-configured`, `manual-match-submit`, `split-submit`), et le `vi.mock` d'`i18n.svelte` des deux
   fichiers de test existants — chaque cas ferait-il rougir la mutation qu'on lui attribue ?
3. **G3 (C-28-2-27)** : le motif `\binstanceof\s+Error\b` rend-il bien les 11 lignes annoncées hors tests (2
   commentaires, 9 de code : 6 dans A/B, 3 dans `console`) ? Les 3 lignes de classe C sont-elles **réellement dans la
   liste d'arguments** d'un appel `console.*` (sinon la garde rougirait à la fin de la story) ?
4. **Planchers et formes de liaison (C-28-2-28, C-28-2-29)** : valeurs « attendu en fin de story » (319, 160, 4, 5,
   169) cohérentes avec la fiche (deux modules neufs, `handleDelete`, aucune liaison retirée) ; les formes typée et
   `async` de `.catch` réellement à 0 aujourd'hui ; la règle « portée `null` = échec » sur les trois chemins est-elle
   écrite sans contradiction avec le paragraphe de l'exemption `console` ?
5. **Montages précisés (AC9, R9/F7/F9)** : `BankProfileList.svelte:26` (`confirm`), `RulesList.test.ts:91`
   (`vi.stubGlobal`), `settings/+page.svelte:248-251` (motif d'A2), `reports/+page.svelte:78` (`data`), `:251`,
   `:254`, `:269`, `:291`, `:309`, `:335`, `:403`, `:437`, `:473`, `:505`, `CancelReconciliationDialog.svelte:60`,
   `:97`, `bank-accounts/+page.svelte:67-78` — chaque ligne citée est-elle exacte ?
6. **Manuel contre code (obligatoire), `.tex` ET PDF aplati** : la phrase neuve du CHANGELOG (AC11) et le retrait de
   la classe C changent-ils ce que le manuel doit dire ? Refais le contrôle de l'AC11 (`tr '\n' ' '` sur les `.tex`,
   `pdftotext … - | tr '\n' ' ' | tr -s ' '` sur les PDF de `docs/manual/fr/`) et compare à la sortie attendue.
7. **Recomptes** écrits par le commit — recompte depuis les fichiers, ne relis pas les nombres : 3 MEDIUM / 18 LOW
   rapportés et 2 / 13 distincts en P2 (rapports F et R) ; 27 mutations (`grep -c "^| M[0-9]"` sur la fiche) ;
   C-28-2-1 à 29 (`grep -c "^### C-28-2-"` sur le registre) ; 53 fichiers importeurs et 166 appels d'`isApiError`
   (commande de R7) ; ligne `28-2-…` de `sprint-status.yaml`.
8. **Propagation** : un symptôme corrigé à un site l'est-il partout (fiche, registre, statut) ? Greper la **valeur**
   (`\b110\b`, `\bsept\b`, `≥ \*\*160`, `B2-B16`, `G1/G3`), pas la formulation.

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — la fiche est une spec, le code n'est pas encore écrit. « `error-message.ts` n'existe pas »,
« `BankProfileList.test.ts` est absent », « aucun test ne soumet `ManualMatchModal` », « `blocEquilibre` n'est pas
exporté », « la clé `error-network` manque aux FTL » sont **attendus**. Un finding ne porte que sur ce que **le texte
du commit affirme du code existant**, ou sur ce qu'il **prescrit de faux ou d'infaisable**.
⚠️ **Entrées passées** : les entrées « Validation P1 » du Change Log et les annotations « Révisé en validation P1 »
gardent leurs chiffres d'alors (consigne 15) — ce n'est pas un résidu.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée ; un
CRITICAL ou HIGH qui affirme qu'un patch « n'a pas été appliqué » se vérifie au `grep -nF` sur le fichier **final**
(`git show eb9c68c8:<chemin>`) avant d'être écrit — les numéros de ligne d'un hunk ne sont pas ceux du fichier final.

## Ce que tu rends

Findings numérotés `P3-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (remédiation P2 / rédaction antérieure), correction proposée. ⛔ **La liste des axes exercés
ET non exercés** (les huit ci-dessus) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-2-validate-p3-ciblee.md` ; dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `docker`, `bash` ou `sh` sur un script, **tout interpréteur** (`python`, `python3`,
`perl`, `ruby`, `node`, `deno`, `awk` avec un programme qui écrit), `gh` en écriture, toute requête SQL. Rien de
`~/kesh-test/` (relevé réel de Guy). Autorisés : lecture, `grep`, `sed -n`, `wc`, `find`, `git log`/`show`/`diff`/
`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
