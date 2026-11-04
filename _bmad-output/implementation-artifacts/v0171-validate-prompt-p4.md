# Prompt — validation P4 de la fiche v0171-correctifs (release corrective 0.17.1), passe ciblée, une lentille, Sonnet

Tu valides la **fiche** de la Story v0171-correctifs de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) — release corrective
0.17.1 (#854, #866, #871, #878, #879), sans migration. Worktree `/home/gcorbaz/devel/kesh-0171`, branche `release/0.17.1`, partie de `910e5d5a`.
Langue du rapport : **français**. Modèle : **Sonnet**. Passe **ciblée** : **une seule lentille**, braquée sur **le seul commit de la remédiation de la validation
P3** (titre : `docs(v0171) : remédiation de la validation P3 (refs #878, refs #879)`). Ce n'est pas une revue de la fiche entière. **Contexte frais.**

⛔ **Piège de l'environnement** : dans ce shell, `grep` est une **fonction** qui appelle `ugrep` (avec `--ignore-files`) et peut rendre autre chose que GNU grep.
Pour toute citation décisive, utilise `/usr/bin/grep` ou `git grep`, et dis-le. Un motif qui rend « rien » se prouve par une sonde positive.

## À lire

1. `git log --oneline -4`, puis `git show <commit de la remédiation P3>` — **c'est ton périmètre**. Le rapport de la P3 :
   `/home/gcorbaz/devel/kesh-gate-logs/v0171-validate-p3.md` (1 MEDIUM, 3 LOW).
2. La fiche `_bmad-output/implementation-artifacts/v0171-correctifs.md` : AC-8 (grep de contrôle), AC-11, AC-12, § 10, § 11 (ligne `products/+page.svelte:308-310`, ligne `vatRate`),
   Change Log (entrées P2 et P3) ; le registre `v0171-choix-autonomes.md` (C-v0171-19, C-v0171-21) ; `sprint-status.yaml` (clé `v0171-correctifs`).
3. Le code : `frontend/src/lib/components/invoices/InvoiceForm.svelte` (`:55-90`, `:105-130`, `:420-430`, `:486-495`, `:600-615`, `:800-815`),
   `frontend/src/routes/(app)/products/+page.svelte:300-315`, `frontend/src/routes/(app)/invoices/due-dates/+page.svelte`,
   `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `…/invoices/reminders/+page.svelte`, `crates/kesh-db/migrations/{20260416000001_invoices,20260428000001_vat_rates}.sql`.
4. Contexte : `CLAUDE.md` du worktree (*La passe ciblée*, *Propagation post-patch*, *Recompter ses propres comptes rendus*).

## Axes à exercer (et seulement ceux-là)

- **AC-12 / AC-8 : le grep de contrôle est-il maintenant décidable ?** Le texte prescrit : motif du § 10 `| /usr/bin/grep -vF formatSwissDate` rend **exactement les cinq lignes**
  d'exception ; pour AC-8, le motif suivi du même filtre rend **0 ligne**. **Rejoue-le** : (1) sur `910e5d5a` (`git grep`/`/usr/bin/grep` sur les huit fichiers de § 10) — combien de lignes avant filtre
  (22 ?), après filtre ? ; (2) **mentalement** sur chaque forme de correction plausible, en particulier : `{inv.paidAt ? formatSwissDate(inv.paidAt.slice(0, 10)) : '—'}` (`due-dates:467`),
  `{formatSwissDate(inv.dueDate ?? '—')}`, `{inv.dueDate ? formatSwissDate(inv.dueDate) : '—'}` (`:1139`, `due-dates:450`, `reminders:345`), une correction écrite sur **deux lignes**
  (la date d'un côté, `formatSwissDate(` de l'autre), `date: formatSwissDate(x.slice(0, 10))` dans un argument i18n. Le critère rend-il la bonne réponse pour chacune ? Le filtre
  peut-il **retirer à tort** l'une des cinq exceptions (une des lignes `:1000, :1014, :1561, :1583`, `due-dates:514` contient-elle `formatSwissDate` ?) ou **laisser** un site corrigé ?
  La liste nominative au Dev Agent Record suffit-elle pour le cas multi-lignes ? Les deux AC (8 et 12) disent-elles la même chose, avec les mêmes chiffres ?
- **`vatRate` (C-v0171-21, § 11)** : la fiche affirme que `vatRate` n'est jamais affiché brut. Lis le code : le `<select>` (`:807`), les options (`{v}%`), `vatOptions` (`:61`), la validation
  (`:492`), l'effet de repli `'8.10'` (`:78-87`) ; les types de colonne (`DECIMAL(5,2)` des deux côtés). Une ligne de facture dont le `vatRate` serveur est d'une autre forme que les options
  (ex. un taux historique absent du magasin, `0.00` contre `0`) produit-elle un `<select>` vide ou une erreur « taux TVA invalide » — un défaut qui relèverait de #878 ou qu'il faudrait tracer ?
  Les numéros de ligne cités sont-ils exacts ? Le registre et le § 11 disent-ils la même chose ?
- **§ 11, `products/+page.svelte:308`** : la phrase remédiée (« zéros de fin retirés quand les deux dernières décimales sont nulles ; `33.3330` garde son zéro ») est-elle exacte au code
  (expression `fixed.replace(/(\.\d{2})0+$/, '$1')`) ? Joue-la sur `100.0000`, `100.2000`, `33.3330`, `33.3300`, `0.0100`.
- **AC-11** : le titre, la formule `readableAmount(<prix lu du serveur>, 2)`, la mention de `p.unitPrice` à l'ajout depuis un article (brouillon ou facture neuve) sont-ils exacts et cohérents
  avec le § 11, T13, M6-ter, le CHANGELOG prescrit et C-v0171-19 ? L'ajout depuis un article s'exécute-t-il bien aussi sur `invoices/new` ?
- **Change Log** : l'entrée P3 est-elle un relevé daté ajouté, sans réécriture d'entrée passée ? L'annotation de l'entrée P2 y est-elle corrigée **par une annotation**, et non par la modification de
  l'entrée P2 (`git diff` du commit : l'entrée P2 est-elle intacte) ? Les entrées antérieures disent-elles encore « deux sites » légitimement (quantité) ?
- **Propagation / régression** : greper la valeur (`exactement les cinq`, `grep -vF`, `trois sites`, `C-v0171-1 à 20` **et** `C-v0171-21` : le registre en porte maintenant **21** — toutes les mentions
  « C-v0171-1 à 20 » vivantes (fiche en-tête, sprint-status, prompts) sont-elles à jour, ou seulement des entrées datées ?) ; le commit a-t-il introduit une incohérence de compte, de renvoi, de numéro ?
- **Hors périmètre** (signale seulement un défaut flagrant) : le fond des AC-1 à AC-7, le manuel, le CHANGELOG de #854, les mutations M1 à M5 et M7, l'inventaire `unitPrice`/`quantity` (validé en P3).

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0171-validate-p4.md` :
1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu, **la ligne lue citée** (sortie de `git grep -n` ou `/usr/bin/grep -nF`,
   copiée) qui fonde l'affirmation, et le remède. Une affirmation de présence ou d'absence sans sortie copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`, `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier
  créé par erreur se **déclare** ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` et `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- `grep` du shell est la fonction `ugrep` : n'en tire aucune citation décisive (`/usr/bin/grep`, `git grep`).
- Commandes permises : `cat`, `sed -n`, `grep` (binaire GNU), `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`, `git diff` (en lecture), `gh issue view`, `gh api` en `GET`.
