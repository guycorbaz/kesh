# Prompt — passe ciblée P3 de la validation de spec, Story 29-5 (à score égal, la facture échue la première)

*Versionné le 2026-10-14. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée — une seule lentille, braquée sur la
dernière remédiation*) : **une** lentille, contexte frais, **Haiku**, sur le **seul** diff de la remédiation P2.
Rotation : P1 Sonnet, P2 Opus, P3 Haiku (D6, rétro 25 : Haiku réservé aux passes ciblées).*

Dépôt `/home/gcorbaz/devel/kesh-29-5`, branche `story/29-5`. **Le périmètre est le commit `688d00b4`**, et lui
seul :

```sh
git -C /home/gcorbaz/devel/kesh-29-5 show 688d00b4
```

C'est un **diff unique** (un commit) : lis-le tel quel, ne le combine avec aucun autre. Ce commit remédie la
validation P2 (rapports `/home/gcorbaz/devel/kesh-gate-logs/29-5-validate-p2-R.md` et `…-p2-F.md`) ; ce qu'il en a
fait est résumé à l'entrée « Validation P2 » du Change Log de la fiche
`_bmad-output/implementation-artifacts/29-5-departage-exigibilite.md`. Le registre touché est
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-5.md` (C-29-5-6, 8, 9, 11 amendés).

⚠️ **Toute affirmation sur le texte se prouve par `grep -nF` sur le fichier FINAL** (dans le worktree, pas dans le
diff) — sortie copiée. Les numéros de ligne d'un diff ne sont pas ceux du fichier final : ne conclus jamais à
l'absence d'un texte sur la foi du diff seul (`CLAUDE.md` § *Haiku-specific guardrails*).

## La lentille — chasseur de régressions de la remédiation P2

La sévérité se déplace vers ce qu'on vient d'écrire. Cherche ce que `688d00b4` a **introduit** ou **laissé à
moitié**, sur ces axes :

1. **La désignation et le client** (MEDIUM P2, R1 = F1) : la fiche dit désormais qu'une facture du **même client**
   au montant exact passe **toujours** devant une facture désignée à quelques centimes près, et qu'une facture d'un
   **autre** client **peut** y passer. Vérifie ces deux énoncés contre la formule `crates/kesh-reconciliation/src/
   matching.rs:80` (`0.50 * amount_score + 0.40 * reference_score + 0.10 * contact_score`), `reference_score`
   (`:121-147`, contenant `:132`, demi-référence `:137-144`) et `contact_score` : « toujours » est-il vrai pour
   **toute** combinaison (contact 0 ou 0,1 de chaque côté, demi-référence de l'autre facture) ? « même client »
   suppose-t-il que le `contact_score` est le même pour les deux factures — et la fiche le dit-elle ? Puis : la
   formulation est-elle **la même** partout où elle est écrite (prémisse, § *Ce que la règle ne fait pas*, doc-comment
   prescrit de l'AC1, texte prescrit du manuel à l'AC7, C-29-5-6) ? Une phrase restée sans condition de client est au
   moins **MEDIUM**.
2. **« devant toute facture qu'il ne désigne pas aussi »** (R3) : exact au code ? reste-t-il un « toute autre » ou un
   « garde la tête » sans cette réserve ?
3. **La macro avant l'ancre** (R4 / F2) : la forme commitée de la 29-2 citée à l'AC1 est-elle celle de son commit
   `8f483388` (`git -C /home/gcorbaz/devel/kesh-29-2 show 8f483388:crates/kesh-db/src/repositories/reconciliation.rs`,
   lignes `:163`, `:180`, `:186`, `:208`) ? La forme composée prescrite a-t-elle des parenthèses et des virgules
   équilibrées, et rend-elle le même texte SQL, au `CAST` près ? ⚠️ L'**arbre de travail** de la 29-2 porte une
   mutation non commitée : lis le **commit**, jamais l'arbre, et n'y touche rien.
4. **M9 en deux régimes** (R5 / F3) : sous M9 avec l'ancre composée, la facture sans échéance est-elle bien exclue
   par le `WHERE` (`{INVOICE_PAYMENT_ANCHOR_SQL} >= ?`, `:399` du commit de la 29-2) ? La liste rendue décrite
   (« la facture `F = D`, puis les quarante-neuf ») a-t-elle le bon nombre d'éléments sous `LIMIT 50` ? Les tests
   annoncés rougir (`:261`, `:314` de `crates/kesh-db/tests/reconciliation_repository.rs`) ont-ils bien des factures
   sans échéance ?
5. **M14 et M15** (F5) : recalcule à la main l'ordre rendu par `…_by_exigibility_before_limit` sous M14
   (`{X} DESC`) — quelle facture est écartée ? La fiche dit « la facture `F = D` » : est-ce exact ? M15 est-elle
   honnêtement écrite comme « à exécuter et à déclarer » ?
6. **AC5, cas 8 et règle d'entrée** (R2 / F4 / R10) : avec `invoice_number: None` ou `"OTHER"` pour les non
   désignées, les scores annoncés (1,0 ; 0,60 ; 0,10 ; 0,50) sont-ils exacts ? Le deuxième sous-cas, « sur une
   transaction sans référence », garde-t-il ses scores ? L'incise « sauf le cas 7 » contredit-elle autre chose ?
7. **Troncature et ensemble** (F7 / R11) : les trois sites (parenthèse d'API, note de contrat, CHANGELOG) et la
   correction de l'entrée de la 29-3 disent-ils la même chose ? La parenthèse d'API reste-t-elle grammaticale dans la
   phrase de la 29-3 (fiche `/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`,
   AC8) ?
8. **R6, R7, R8, F6** : `repositories/reconciliation.rs:210-212` et `tests/reconciliation_repository.rs:304-305`,
   `:378` disent-ils ce que la fiche leur prête ? `routes/reconciliation.rs:140` porte-t-il `invoice_date` ?
   `kesh-core/src/lettering/proposals.rs` et `repositories/letterings/open_items.rs:531-532` disent-ils ce que
   l'inventaire (b) leur prête ?
9. **Manuel** (obligatoire, `CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) : le texte prescrit du
   manuel (AC7) — modifié par ce commit — promet-il quelque chose que le code prescrit ne tiendrait pas ? Le PDF au sol
   (`docs/manual/fr/user-manual.pdf`) n'est pas encore régénéré : contrôle seulement que rien, au sol, ne contredit
   déjà le texte prescrit — `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "mieux notée[^.]*"`.
10. **Recomptes** de l'entrée « Validation P2 » du Change Log : 15 mutations (`grep -cE '^\| M[0-9]+ \|'`), 9 AC,
    7 tâches, 9 cas, 6 lignes, 12 choix, 3 tests neufs ; les bilans 0/0/1/10 et 0/0/1/7 contre les rapports P2 ;
    chaque finding des rapports P2 (R1-R11, F1-F8) traité dans l'entrée.

**Hors périmètre** : tout ce que `688d00b4` ne touche pas. Si tu y vois un défaut, signale-le en une ligne, sans
sévérité, sous « Hors périmètre ».

## Ce que tu rends

Findings numérotés `P3-1…`, sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne` **du fichier final**,
**preuve** (commande exécutée et sortie copiée — `grep -nF` obligatoire pour toute présence ou absence), correction
proposée. ⛔ **La liste des axes exercés ET non exercés** (1 à 10 ci-dessus) — un « 0 finding » sans elle ne compte
pas comme passe, et un axe « survolé » se déclare non exercé. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/29-5-validate-p3-ciblee.md` ; dernier message : le chemin du rapport, le bilan
par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits — lecture seule stricte

⛔ N'écris **aucun** fichier hors du rapport ci-dessus. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `cargo`, `rustc`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, `mariadb`/`mysql` et toute requête SQL, `gh` en
écriture, `sed -i`. **Aucune** commande `git` autre que `show`, `log`, `diff`, `blame`, `status` — en particulier
**ni `checkout` sous aucune forme** (`--detach` compris), ni `switch`, `stash`, `fetch`, `pull`, `reset`, `rebase`,
`commit`, `add`, `tag`, `push`, `worktree`. Les worktrees `kesh-29-2` et `kesh-29-3` se lisent et ne se modifient
pas. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`,
`git show`/`log`/`diff`/`blame`/`status`, `pdftotext` vers la sortie standard.
