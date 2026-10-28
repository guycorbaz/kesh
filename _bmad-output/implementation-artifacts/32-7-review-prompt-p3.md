# Revue de code P3 — Story 32-7 — passe ciblée : la remédiation de la passe 2

Prompt versionné. **Passe ciblée**, une seule lentille : **R — chasse aux régressions**. Modèle : **Haiku**, contexte
frais, lecture seule.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-7`. La passe 2 (Sonnet, ciblée ; rapport
`/home/gcorbaz/devel/kesh-gate-logs/327-review-p2.md`) a rendu 0/0/1/4. Sa remédiation est le **seul** objet de cette
passe — un seul diff, aplati :

```
git -C /home/gcorbaz/devel/kesh-32-10a diff f45e343e..HEAD
```

Ce qu'elle fait, à contester :

- **P2-1 (MEDIUM)** — `frontend/src/lib/features/mcp-server/McpKeysSection.test.ts` : un test neuf, « le 31.12.9999
  sous un fuseau négatif (au-delà de 9999 en UTC) est refusé à l'écran », qui pose `process.env.TZ =
  'America/New_York'` et le rétablit dans un `finally`, avec deux témoins (décalage 300 ; bascule de
  `toISOString()` en `+010000-`). Il doit tuer la mutation M36 (le seul disjoint `!/^\d{4}-/.test(expiresAt)` retiré
  de `McpKeysSection.svelte`) : journal `/home/gcorbaz/devel/kesh-gate-logs/327/327-mut-M36.log`. Aucun code de
  production n'est touché (C-32-7-39 au registre `epic-32-choix-autonomes-327.md`).
- **P2-2** — `crates/kesh-i18n/locales/it-CH/messages.ftl` : `mcp-server-secret-created = Chiave «{ $name }» creata.`
- **P2-5** — `crates/kesh-i18n/locales/de-CH/messages.ftl` : `mcp-server-scope-invoices`, « der Fälligkeitsplan »
  (nominatif).
- **P2-4** — `docs/manual/fr/admin-manual.tex` (§ *Le filtre d'adresse*, `sec:mcp-filtre`) : « en plage admise comme
  en mandataire de confiance (dans ce dernier cas, filtre activé seulement) » ; PDF régénéré
  (`docs/manual/fr/admin-manual.pdf`).
- Change Log de la fiche (`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`, entrée « revue de code
  P2 ») et registre (C-32-7-39).

## Ce que la lentille cherche

1. **Le test neuf passe-t-il à vide ?** Le fuseau prend-il vraiment (le témoin le prouve-t-il) ? Le `finally`
   rétablit-il `TZ` pour les tests suivants du même fichier, même en cas d'échec ? Le test vise-t-il bien le second
   disjoint, et non le premier (`expiresAt === null`) ? La mutation M36 est-elle tuée par **ce** test (journal) ?
2. **Ce que la remédiation casse** : un autre test du fichier qui dépendait du fuseau par défaut ; une valeur de
   catalogue dont la forme change le rendu (`{ $name }` toujours présent, mêmes variables que fr-CH) ; une phrase du
   manuel qui ne dit plus vrai au code (`McpSettingsSection.svelte` : l'avertissement des mandataires ne paraît que
   filtre activé ; celui des plages toujours).
3. **Le symptôme ailleurs** : d'autres guillemets « … » espacés dans les clés `mcp-server-` it/de ; d'autres
   accusatifs/nominatifs mêlés dans les phrases de portée de-CH.
4. **Le PDF** : la phrase récrite y est-elle, l'ancienne n'y est-elle plus ? Lire le PDF aplati
   (`pdftotext /home/gcorbaz/devel/kesh-32-10a/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
   ⚠️ **Jamais de `grep -oE '…{0,N}'` sur la ligne aplatie entière** (deux tels grep ont pris 1,7 Go chacun et ont dû
   être tués) : seulement `grep -cF` ou `grep -oF` sur des motifs **fixes**, chacun sous `timeout 60`, par
   `timeout 60 /usr/bin/grep -cF '…'` (sur ce poste, `grep` est une fonction du shell que `timeout` ne lance pas).

## ⛔ Exigence de preuve

**Toute affirmation de présence ou d'absence** — d'un code, d'un test, d'une clé, d'une phrase — **s'appuie sur une
sortie `grep -nF` copiée dans le rapport** (`command grep -nF '<chaîne exacte>' <fichier>`), ou, pour le PDF, sur la
sortie `timeout 60 /usr/bin/grep -cF` ci-dessus. Lis le diff **aplati** ci-dessus, jamais la suite des commits : un
numéro de ligne d'un diff intermédiaire ne vaut pas pour le fichier final. Une affirmation sans sortie copiée ne
compte pas.

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour. N'exécute aucun script du
  dépôt, ni `cargo`, `npm`, `npx`, `node`, `python3`, `sqlx`, `docker`, `mariadb`/`mysql`, `make`/`latexmk`, `curl`.
- **Aucune redirection** (`>`, `>>`, `<`, `tee`, here-doc) ; **ni `sed -i`, ni `perl -pi`, ni `rm` sous aucune
  forme.**
- N'écris, ne modifie, ne crée aucun fichier — **sauf ton rapport**, écrit avec l'outil d'écriture de fichiers. Aucune
  commande git qui écrit, **aucun `git checkout`/`git switch`**, ni `stash`, `reset`, `rebase`, `tag`, `push` ; aucune
  commande `gh` qui écrit. Aucun `pkill`/`kill`. N'ouvre jamais `~/kesh-test/`.
- Commandes autorisées : `git diff`, `git show`, `git log`, `command grep`, `/usr/bin/grep`, `sed -n`, `cat`,
  `pdftotext` (vers `-`), `tr`, `timeout`, lecture de fichiers.

## Rapport

Écris-le dans `/home/gcorbaz/devel/kesh-gate-logs/327-review-p3.md` :

1. **Axes exercés** et **axes NON exercés** — liste explicite. Un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `P3-n`, chacun avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne, description,
   preuve (sortie `grep -nF` copiée), correctif proposé. Un défaut hors du diff se signale comme tel (« hors
   périmètre, à tracer »).
3. Verdict : nombre de findings par sévérité.
