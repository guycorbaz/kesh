#!/bin/bash

# ============================================================================
# init-demo.sh — Initialize Kesh demo instance via database seeding
# ============================================================================
#
# Creates a fully initialized demo company with:
# - Company with all accounts (PME chart)
# - Admin user (admin / admin123)
# - Onboarding state marked as complete
# - Company invoice settings configured
#
# Usage:
#   ./init-demo.sh                    # Use default database
#   ./init-demo.sh custom-container   # Use specific container name

set -e  # Exit on error

# Configuration
CONTAINER="${1:-kesh-mariadb}"
DB_NAME="kesh"
DB_USER="kesh"
DB_PASS="kesh_dev"

# Argon2id hash for password "admin123" (matching test_fixtures.rs)
ADMIN_PASSWORD_HASH='$argon2id$v=19$m=19456,t=2,p=1$wDaFUbAJuozHKhQshibCHw$T/DeYTKABHDpW7JM5MoiQciUad5Eb81Cfvh0aUvi2Z4'

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# ============================================================================
# Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}ℹ ${1}${NC}"
}

log_success() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠ ${1}${NC}"
}

log_error() {
    echo -e "${RED}✗ ${1}${NC}"
}

check_container() {
    log_info "Checking database container..."
    if ! docker ps | grep -q "$CONTAINER"; then
        log_error "Container '$CONTAINER' is not running"
        log_info "Start with: docker compose up -d"
        exit 1
    fi
    log_success "Container is running"
}

seed_database() {
    log_info "Seeding database with demo company..."

    # Use sed to replace the hash placeholder in the SQL
    cat <<'EOSQL' | sed "s|\$ADMIN_PASSWORD_HASH|$ADMIN_PASSWORD_HASH|" | docker exec -i "$CONTAINER" mariadb -u "$DB_USER" -p"$DB_PASS" "$DB_NAME"
-- Delete existing data (if any) to ensure clean state
DELETE FROM company_invoice_settings WHERE company_id IN (SELECT id FROM companies);
DELETE FROM onboarding_state;
DELETE FROM users WHERE company_id IN (SELECT id FROM companies);
DELETE FROM accounts WHERE company_id IN (SELECT id FROM companies);
DELETE FROM companies;

-- Create company
INSERT INTO companies (name, address, org_type, accounting_language, instance_language, version, created_at, updated_at)
VALUES ('Demo Company', 'Demo Address', 'Independant', 'FR', 'FR', 1, NOW(), NOW());

SET @company_id = LAST_INSERT_ID();

-- Create onboarding state (marked as complete, demo mode)
INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode, version, created_at, updated_at)
VALUES (TRUE, 3, TRUE, 'guided', 1, NOW(), NOW());

-- Create accounts (PME chart)
INSERT INTO accounts (company_id, number, name, account_type, active, version, created_at, updated_at)
VALUES
    (@company_id, '1000', 'Caisse', 'Asset', 1, 1, NOW(), NOW()),
    (@company_id, '1100', 'Créances', 'Asset', 1, 1, NOW(), NOW()),
    (@company_id, '1200', 'Stocks', 'Asset', 1, 1, NOW(), NOW()),
    (@company_id, '1300', 'Matériel', 'Asset', 1, 1, NOW(), NOW()),
    (@company_id, '2000', 'Dettes', 'Liability', 1, 1, NOW(), NOW()),
    (@company_id, '2100', 'Crédits', 'Liability', 1, 1, NOW(), NOW()),
    (@company_id, '3000', 'Ventes', 'Revenue', 1, 1, NOW(), NOW()),
    (@company_id, '4000', 'Charges', 'Expense', 1, 1, NOW(), NOW()),
    (@company_id, '4100', 'Personnel', 'Expense', 1, 1, NOW(), NOW()),
    (@company_id, '4200', 'Autres charges', 'Expense', 1, 1, NOW(), NOW());

-- Create admin user with Argon2 hash
INSERT INTO users (company_id, username, password_hash, role, active, version, created_at, updated_at)
SELECT
    @company_id,
    'admin',
    '$ADMIN_PASSWORD_HASH',
    'Admin',
    1,
    1,
    NOW(),
    NOW();

-- Create company_invoice_settings with default accounts
INSERT INTO company_invoice_settings (
    company_id,
    invoice_number_format,
    default_receivable_account_id,
    default_revenue_account_id,
    default_sales_journal,
    journal_entry_description_template,
    version,
    created_at,
    updated_at
)
SELECT
    @company_id,
    'F-{YEAR}-{SEQ:04}',
    (SELECT id FROM accounts WHERE company_id = @company_id AND number = '1100' LIMIT 1),
    (SELECT id FROM accounts WHERE company_id = @company_id AND number = '3000' LIMIT 1),
    'Ventes',
    '{YEAR}-{INVOICE_NUMBER}',
    1,
    NOW(),
    NOW();

-- Verify data was created
SELECT 'Demo company created' as status;
SELECT COUNT(*) as account_count FROM accounts WHERE company_id = @company_id;
SELECT username FROM users WHERE company_id = @company_id;
EOSQL

    log_success "Database seeded successfully"
}

test_api() {
    log_info "Testing API access..."

    local health=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health)
    if [ "$health" = "200" ]; then
        log_success "API is healthy"
        return 0
    else
        log_warning "API health check returned: $health"
        return 1
    fi
}

# ============================================================================
# Main
# ============================================================================

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Kesh — Initialize Demo Instance                           ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

check_container
echo ""

seed_database
echo ""

test_api
echo ""

# Final summary
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  ✓ Demo initialization complete!                           ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Login credentials:"
echo "  Username: ${BLUE}admin${NC}"
echo "  Password: ${BLUE}admin123${NC}"
echo ""
echo "Access the application:"
echo "  ${BLUE}http://localhost:3000${NC}"
echo ""
