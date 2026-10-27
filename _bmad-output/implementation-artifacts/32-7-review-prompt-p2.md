# Revue de code P2 — Story 32-7 — passe ciblée : la remédiation de la passe 1

Prompt versionné. **Passe ciblée**, une seule lentille : **R — chasse aux régressions**. Modèle : **Sonnet**, contexte
frais, lecture seule.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-7`. La passe 1 (Opus, B/E/A ; rapports
`/home/gcorbaz/devel/kesh-gate-logs/327-review-p1-{A,B,E}.md`) a rendu A 0/0/0/7, B 0/0/0/8, E 0/0/3/4. Sa remédiation
est le **seul** objet de cette passe :

```
git -C /home/gcorbaz/devel/kesh-32-10a diff f9b154f8..HEAD
```

(aplati : un seul diff, du dernier commit de la passe de développement à `HEAD`.) Ce qu'elle fait, à contester :

- **E1** — `McpKeysSection.svelte` : `{#each k.unknownScopes as u, i (i)}` (une portée inconnue stockée deux fois
  faisait lever `each_key_duplicate`).
- **E2 = B4** — `mcp-reglages.ts` : `familleEtPrefixe` lit famille et préfixe comme `reseau_canonique`
  (`crates/kesh-api/src/mcp/reseau.rs:39-49`) ; `aPrefixeNul` et `familleDe` en dérivent ; `groupesIpv6` reconnaît les
  graphies d'une adresse mappée (C-32-7-34). Manuel `sec:mcp-filtre` récrit.
- **E3** — `McpSettingsSection.svelte` : rien de l'état (case, phrase, filtre, listes, avertissements, date) ne paraît
  tant que les réglages ne sont pas lus ; `<fieldset disabled={envoi}>` ; bouton *Réessayer* ; `charger()` rend un
  booléen (C-32-7-35).
- **LOW** pris : B1 (fuseau négatif au test des dates), B2 (secret absent → erreur, liste relue, C-32-7-37), B3
  (expiration illisible ou au-delà de 9999 refusée, C-32-7-36), B5 (traductions it/de, guillemets allemands), B8
  (succès effacé à la saisie ; « rechargés » après relecture réussie seulement ; champs figés pendant l'envoi), A-1
  (annotation de la fiche), A-3 (`PORTEES_MCP` comparé à `mcp_key.rs:48-54`), A-5 (sonde du créateur), A-6 (recette
  SQL du manuel), A-7 (secret factice hors format, C-32-7-38).
- Clés i18n neuves `mcp-server-settings-retry`, `mcp-server-form-expires-invalid` (quatre locales) ; G13 par préfixe
  95 → 97 ; `sitesTotal` 2617 → 2619.

Journaux : `/home/gcorbaz/devel/kesh-gate-logs/327/` (`rouge-remediation-p1.log`, mutations M27 à M35
`327-mut-M<n>.log`, `mutations-campagne-p1.log`, gates `*-p1*.log`, `pdf-motifs-3.log`) ; captures refaites
`/home/gcorbaz/devel/kesh-gate-logs/327-captures/p1-remediation/`.

## Ce que la lentille cherche

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire*. Pour chaque hunk du diff :
1. **ce que le correctif casse** — un cas qui marchait avant (ex. : une entrée IPv4 ordinaire, `::/0`, une adresse sans
   préfixe, un préfixe non numérique, une entrée illisible, `::ffff:192.168.1.10` sans préfixe, un `/96` non mappé ;
   un test existant devenu muet parce que les champs n'apparaissent plus pendant le chargement ; le `<fieldset>` qui
   change le rendu ou les sélecteurs E2E ; `charger()` booléen et ses autres appelants) ;
2. **ce que le correctif ne couvre qu'à moitié** — `groupesIpv6` contre les graphies réelles d'IPv6 (majuscules,
   zéros de tête, `::` en tête ou en fin, queue pointée hors d'une adresse mappée, plus de huit groupes) comparée à la
   lecture de `IpNet` côté serveur ; le symptôme corrigé grepé ailleurs (autres `{#each … (valeur)}` sur des données du
   serveur, autres `String(…)` sur une réponse) ;
3. **le manuel et le PDF** : la phrase récrite de `sec:mcp-filtre` et la recette SQL disent-elles vrai au code
   (`reseau.rs`, `routes/mcp_server_settings.rs`) ? Lire le PDF aplati
   (`pdftotext /home/gcorbaz/devel/kesh-32-10a/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
   ⚠️ **Jamais de `grep -oE '…{0,N}'` sur la ligne aplatie entière** (deux tels grep ont pris 1,7 Go chacun et ont dû
   être tués) : `command grep -cF` ou `command grep -oF` sur des motifs **fixes**, chacun sous `timeout 60`
   (`timeout 60 /usr/bin/grep -cF '…'`, `timeout` ne lançant pas la fonction `grep` du shell) ;
4. **les tests et mutations neufs** : chaque test passe-t-il à vide (témoin absent, `waitFor` satisfait avant
   l'état visé) ? chaque mutation M27-M35 est-elle tuée par le test visé ? les traductions it/de sont-elles justes ?

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour. N'exécute aucun script du
  dépôt, ni `cargo`, `npm`, `npx`, `node`, `python3`, `sqlx`, `docker`, `mariadb`/`mysql`, `make`/`latexmk`, `curl`.
- **Aucune redirection** (`>`, `>>`, `<`, `tee`, here-doc) ; **ni `sed -i`, ni `perl -pi`, ni `rm` sous aucune
  forme.**
- N'écris, ne modifie, ne crée aucun fichier — **sauf ton rapport**, écrit avec l'outil d'écriture de fichiers. Aucune
  commande git qui écrit, **aucun `git checkout`/`git switch`**, ni `stash`, `reset`, `rebase`, `tag`, `push` ; aucune
  commande `gh` qui écrit. Aucun `pkill`/`kill`. N'ouvre jamais `~/kesh-test/`.
- Commandes autorisées : `git diff`, `git show`, `git log`, `command grep`, `sed -n`, `cat`, `pdftotext` (vers `-`),
  `tr`, `timeout`, lecture de fichiers et d'images.

## Rapport

Écris-le dans `/home/gcorbaz/devel/kesh-gate-logs/327-review-p2.md` :

1. **Axes exercés** et **axes NON exercés** — liste explicite. Un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `P2-n`, chacun avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne, description,
   preuve (sortie `command grep -nF` copiée ; pour le manuel, le PDF aplati), correctif proposé. Un défaut hors du diff
   se signale comme tel (« hors périmètre, à tracer »).
3. Verdict : nombre de findings par sévérité.
