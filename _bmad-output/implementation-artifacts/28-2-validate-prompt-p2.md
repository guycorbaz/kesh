# Prompt — validation P2 de la spec, Story 28-2 (une erreur d'API ne s'affiche plus « [object Object] »)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; **P2 : Opus**
pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-28-2`, branche `story/28-2` (base `6f971d76`, = v0.13.2 + rétrospective de l'Epic 27).
Fiche : `_bmad-output/implementation-artifacts/28-2-erreurs-object-object.md`. Issue #520 et issue **#644** (classe D,
ouverte par l'orchestrateur) : `gh api repos/guycorbaz/kesh/issues/520`, `…/520/comments`, `…/644`. Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-2.md` (C-28-2-1 à 24 ; **15 à 24 sont neufs**).
Rapports de la P1 (à lire, hors dépôt) : `/home/gcorbaz/devel/kesh-gate-logs/28-2-validate-p1-F.md` et
`28-2-validate-p1-R.md`. Remédiation à relire : le commit `256fdc9f`, `git diff 69177d6b 256fdc9f --
_bmad-output/implementation-artifacts/28-2-erreurs-object-object.md
_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-2.md` (diff aplati, à préférer). Antécédent : `15-5b-gardes-surfaces-neuves.md` (AC14, C-15-5b-1, `:976-979`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — la passe ciblée, le manuel nommé, les axes
déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting
préventif et son amendement D5 ; § Test Locally First, E2E ; § Issue Tracking Rule).

⚠️ **Décision de l'orchestrateur, à ne pas rejuger** : la classe D (F4, R2, R3 de la P1) reste **hors périmètre**,
suivie par #644. Juge seulement si la fiche la traite **correctement** comme angle mort (renvoi `refs #644`, aucune
issue de suivi supplémentaire prescrite, aucun reste d'une rédaction qui la promettait à une issue « à ouvrir »).
Une recommandation d'intégrer la classe D n'est pas un finding.

## Axe premier — les régressions de la remédiation P1

*La sévérité se déplace vers ce qu'on vient d'écrire* (CLAUDE.md, § La passe ciblée). Avant toute chose, relis
le diff aplati de la remédiation hunk par hunk et cherche ce que **la remédiation a cassé ou affirmé à tort** :

1. **Chaque fait neuf se revérifie au sol**, `sed -n` / `grep -nF` à l'appui : `user-manual.tex:1871-1873`,
   `admin-manual.tex:1472`, `i18n.svelte.ts:48-50` (le `catch {}` silencieux de `loadI18nMessages`),
   `routes/+layout.svelte:36-53`, `routes/login/+page.svelte:50`, `BankProfileList.svelte:25-31` et `:41`,
   `invoices/[id]/+page.svelte:186`, `:483`, `:1111`, `:1113`, `:411`, `journal-entries/[id]/+page.svelte:89`,
   `ProductPicker.svelte:55`, `BankAccountJournalLinkForm.svelte:11`, `:95`, `:124`, `auth.svelte.test.ts:196`,
   `i18n-literal-reader.js:141`, `:183`, `:211`, `:408`, `:413-418`, et `i18n-literal-reader.test.ts:310-331`.
   La fiche prescrit de généraliser `corpsDeFonction` en `blocEquilibre(source, ouvrante, ouvre, ferme)` exportée,
   parce que les outils de saut des littéraux sont privés : la généralisation est-elle sûre pour les **trois**
   appelants existants (`i18n-libelle-en-dur.test.ts:341`, `:530`, et le lecteur lui-même) — le saut des regex et
   des chaînes vaut-il tel quel pour des parenthèses (une parenthèse dans une regex, `/\(/`) ?
2. **Les décomptes neufs se recomptent** : 256 `catch` hors commentaires, 163 liés, 4 typés, 159 non typés, 5
   `.catch` liés, 168 liaisons ; 11 lignes `instanceof Error` dont 2 commentaires et 9 de code ; 7 MEDIUM / 11 LOW de
   la P1 ; C-28-2-15 à 24 ; M21-M25 ; les sorties « attendues » du contrôle aplati des manuels (AC11) — exécute la
   commande `pdftotext … | tr …` sur les PDF et compare.
3. **Les prescriptions neuves sont-elles justes et cohérentes entre elles ?**
   - G3 « toute forme, dans tout le fichier » : quelle occurrence légitime de `instanceof Error` (hors tests, hors
     commentaires, hors `console`) la règle ferait-elle rougir à la fin de la story — `error-message.ts`, un futur
     utilitaire, `pdf-error.ts`, `download.ts` ? La promesse « les 9 lignes de code disparaissent » tient-elle site
     par site ?
   - C-28-2-19 (journaliser la valeur brute) : un test existant asserte-t-il les arguments de ces `console.*` ? La
     console du navigateur et Vitest rendent-ils réellement l'objet ? Le changement modifie-t-il un comportement
     (hydratation, #347) ?
   - C-28-2-20 (`handleDelete`) : le `try` englobe-t-il `load()`, et est-ce voulu ? `notifyError` est-il importable
     sous les `vi.mock` du test prescrit ? « Vitest fait échouer le fichier sur un rejet non géré » est-il vrai dans la
     configuration du dépôt (`frontend/vite.config.ts`, `dangerouslyIgnoreUnhandledErrors`, etc.) ?
   - Les planchers par forme (C-28-2-17) : la story elle-même retire-t-elle une liaison (A4 est un `.catch` lié,
     `handleDelete` en **ajoute** un `catch`) — les mesures et planchers écrits survivent-ils à la story ? Le total
     « ≥ 160 » est-il encore utile, ou redondant ?
   - La portée des liaisons (C-28-2-16) : le calcul « de `=>` à la parenthèse qui ferme `.catch(` » est-il correct
     pour `.catch((err) => (cond ? a : String(err)))`, pour un `.catch` suivi d'un `.finally(…)`, et dans un
     `.svelte` dont le balisage contient des `{…}` ? L'auto-test au gabarit imbriqué est-il du JS valide ?
   - AC9 / C-28-2-21 : les deux tests de page neufs (`reports`, `api-keys`) sont-ils montables (le `+page.ts` de
     `reports`, ses dépendances, ses `vi.mock`) ? Les mutations M21-M23 les font-elles réellement rougir ?
   - AC10 : la phrase « M5 ne la fait pas rougir » est-elle exacte ? AC11 : la phrase du CHANGELOG est-elle vraie
     (page de connexion **et** navigation après connexion) et exacte en français ?
4. **Propagation** : chaque symptôme corrigé (le zéro du manuel, 257/164/169, le plancher 300, « issue de suivi
   demandée », « langue de l'installation », classe C = 3, l'exception `reports` de l'AC9) a-t-il été corrigé
   **partout** dans la fiche et le registre — Status, Story, AC, Tasks, Dev Notes, angles morts, mutations — sans
   réécrire le Change Log passé (consigne 15) ? Greppe la **valeur**, pas la formulation.

## Axes communs (repris de la P1, en complet)

- **Inventaire clos** (§ *Inventaire des sites NON RÉSOLUS*) : refais et élargis les `grep` (noms de liaison,
  `catch (e: unknown)`, `.catch(function (x)`, second argument de `.then`, `allSettled`/`.reason`, `{:catch}`,
  `onerror`, `unhandledrejection`, `+error.svelte`, `String(` dans le balisage, `toast.error(`/`notifyError(` recevant
  une valeur brute, concaténations, casts `as Error|ApiError|any`, **et les rejets sans aucun `catch`** — la forme de
  `handleDelete`, qu'aucun détecteur ne voit : en reste-t-il dans les fichiers que la story touche ?). Un site
  affiché qui rend « [object Object] » et manque à la classe A est au moins **HIGH** ; un site de classe B manquant,
  au moins **MEDIUM**.
- **Manuel contre code** (obligatoire) : `.tex` **et** PDF aplatis (`pdftotext docs/manual/fr/user-manual.pdf - |
  tr '\n' ' ' | tr -s ' '`, idem `admin-manual.pdf`, `marketing-brochure.pdf`), ce que le manuel dit du message d'un
  refus au verrouillage des livres et aux profils bancaires, et la suppression d'un profil bancaire (C-28-2-20 change
  ce que voit l'utilisateur : le manuel en parle-t-il ?). Une promesse que le code prescrit ne tiendrait pas est au
  moins MEDIUM.
- **Recomptes** : tout nombre de la fiche, y compris ceux que la P1 n'a pas touchés (26 lignes et leur ventilation,
  4 + 16, ≈ 23 classe D, 177 lectures de `.message`, 20 sites `error-unexpected`, 18 `vi.mock` du client,
  `sitesTotal` 2075 → 2079 — `handleDelete` n'ajoute-t-il vraiment aucun site `i18nMsg` ?), 11 AC, T0-T10, M1-M25.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #520 couverte par un AC testable, chaque AC par une tâche et par un test **qui rougirait** ;
  chaque mutation M1-M25 confrontée au test prescrit ; chaque fait, chemin et numéro de ligne revérifié ; les textes
  prescrits (trois clés × 4 locales, CHANGELOG) contre le glossaire et les traductions voisines ; les gardes
  existantes — dont les trois que la P1 a ajoutées à l'AC7 (`i18n-repli-divergent-actif.test.ts`,
  `i18n-harvest.test.ts`, `i18n-dette-connue.ts`) — : leur rôle est-il décrit **exactement** ?
- **F — Faisabilité et conception (adversaire)** : la garde AC6 telle que réécrite est-elle **implémentable** avec ce
  qu'exporte `i18n-literal-reader.js`, décidable, sans faux négatif sur les vingt sites et sans faux positif à la
  fin de la story ; ses planchers et auto-tests détectent-ils chacune des mutations M16-M18, M24, M25 ? Le cycle
  d'import `api-client.ts` ↔ `i18n.svelte.ts` (validé bénin en P1) est-il encore bénin avec ce que la remédiation
  ajoute (`notifyError` dans `BankProfileList`, `error-message.ts`) ? La dérogation au découpage, réexaminée
  (septième module `app`, D5), tient-elle ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chacun, dis s'il vient **de la
remédiation P1** ou de la rédaction d'origine. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme
passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-2-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-2-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/install-hooks.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `deno`, `bun`,
`python`/`python3`, `perl`, `ruby`, `tsc`, `vitest`, `playwright`, `docker`, `bash`/`sh` sur un script du dépôt,
**tout interpréteur ou exécutable de script, quel qu'il soit**, `gh` en écriture (`pr`, `issue`, `comment`,
`api -X POST/PATCH/PUT/DELETE`), toute requête SQL, `sed -i`. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit
ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `find`, `wc`, `ls`, `git log`/`show`/`diff`/`blame`/`grep`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
