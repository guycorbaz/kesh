#!/bin/bash
M="docker exec -i kesh-mariadb-dev mariadb -uroot -pkesh_dev_root kesh -N"
$M -e "INSERT INTO fiscal_years (company_id,name,start_date,end_date,status) VALUES (1,'T2024','2024-01-01','2024-12-31','Closed'),(1,'T2025','2025-01-01','2025-12-31','Open'),(1,'T2027','2027-01-01','2027-12-31','Open');" 2>/dev/null
IDS=$($M -e "SELECT GROUP_CONCAT(id ORDER BY start_date) FROM fiscal_years WHERE company_id=1" 2>/dev/null)
probe() {
  label="$1"; q="$2"
  ( echo "BEGIN; $q; SELECT SLEEP(4); ROLLBACK;" | $M >/dev/null 2>&1 ) &
  sleep 1.5
  out="$label :"
  for id in ${IDS//,/ }; do
    sd=$($M -e "SELECT start_date FROM fiscal_years WHERE id=$id" 2>/dev/null)
    r=$(echo "BEGIN; SELECT id FROM fiscal_years WHERE id=$id FOR UPDATE NOWAIT; ROLLBACK;" | $M 2>&1 | grep -q "ERROR" && echo "VERROUILLÉ" || echo "libre")
    out="$out $sd=$r"
  done
  echo "$out"; wait
}
probe "premier, ORDER BY start_date" "SELECT id FROM fiscal_years WHERE company_id = 1 ORDER BY start_date LIMIT 1 FOR UPDATE"
probe "premier, ORDER BY start_date, id" "SELECT id FROM fiscal_years WHERE company_id = 1 ORDER BY start_date, id LIMIT 1 FOR UPDATE"
probe "candidat du jour, DESC" "SELECT id FROM fiscal_years WHERE company_id = 1 AND start_date <= '2026-10-06' ORDER BY start_date DESC LIMIT 1 FOR UPDATE"
probe "find_open_covering_date" "SELECT id FROM fiscal_years WHERE company_id = 1 AND start_date <= '2026-10-06' AND end_date >= '2026-10-06' AND status='Open' LIMIT 1 FOR UPDATE"
$M -e "DELETE FROM fiscal_years WHERE company_id=1 AND name IN ('T2024','T2025','T2027'); SELECT COUNT(*) FROM fiscal_years;" 2>/dev/null
