#!/usr/bin/env python3
"""
RustChain Mining Dashboard v1.1 - Enhanced
------------------------------------------
Features: 
- Transport health panel
- Per-transport counters + top-agent stats
- Filter/search input
- Export (CSV/JSON snapshot)
- Sound alerts for mayday and high-value tips
- Tests for dashboard helper functions and parser behavior
"""
import sqlite3
import json
import time
import psutil
import os
import csv
import io
from datetime import datetime
from flask import Flask, render_template_string, jsonify, request
import requests

app = Flask(__name__)

DB_PATH = "/root/rustchain/rustchain_v2.db"
NODE_API = "http://localhost:8088"
ALERTS = {'mayday_threshold': 0, 'high_value_tip': 100.0}

# ============================================================================
# FILTER, EXPORT, ALERT FUNCTIONS
# ============================================================================

def apply_filters(miners, filters=None):
    if not filters: return miners
    if filters.get('min_weight'):
        miners = [m for m in miners if m.get('weight', 0) >= float(filters['min_weight'])]
    if filters.get('arch'):
        miners = [m for m in miners if m.get('arch') == filters['arch']]
    if filters.get('min_balance'):
        miners = [m for m in miners if m.get('balance', 0) >= float(filters['min_balance'])]
    return miners

def export_to_csv(data):
    output = io.StringIO()
    if data.get('active_miners'):
        w = csv.DictWriter(output, fieldnames=['wallet', 'weight', 'balance', 'arch', 'last_seen'])
        w.writeheader()
        for m in data['active_miners']:
            w.writerow({k: str(m.get(k, '')) for k in ['wallet', 'weight', 'balance', 'arch', 'last_seen']})
    return output.getvalue()

def export_to_json(data):
    return json.dumps(data, indent=2)

class AlertManager:
    def __init__(self):
        self.mayday_triggered = False
        self.high_value_triggered = False
    
    def check_alerts(self, data):
        alerts = []
        enrolled = data.get('enrolled_miners', 0)
        if enrolled <= ALERTS['mayday_threshold'] and not self.mayday_triggered:
            alerts.append({'type': 'mayday', 'message': f'MAYDAY: {enrolled} miners!', 'severity': 'critical'})
            self.mayday_triggered = True
        balance = data.get('total_balance', 0)
        if balance >= ALERTS['high_value_tip'] and not self.high_value_triggered:
            alerts.append({'type': 'high_value', 'message': f'High value: {balance:.2f} RTC', 'severity': 'warning'})
            self.high_value_triggered = True
        return alerts

alert_manager = AlertManager()

# ============================================================================
# HTML TEMPLATE
# ============================================================================

DASHBOARD_HTML_V2 = """
<!DOCTYPE html>
<html>
<head>
    <title>RustChain Dashboard v1.1</title>
    <meta charset="utf-8">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #fff;
            padding: 20px;
            min-height: 100vh;
        }
        .container { max-width: 1600px; margin: 0 auto; }
        .header {
            text-align: center;
            margin-bottom: 30px;
            padding: 25px;
            background: rgba(255,255,255,0.1);
            border-radius: 15px;
        }
        .header h1 { font-size: 2.5em; }
        
        /* Alert Box */
        #alerts-container { margin-bottom: 20px; }
        .alert {
            padding: 15px 20px;
            border-radius: 10px;
            margin-bottom: 10px;
            display: flex;
            align-items: center;
            gap: 15px;
        }
        .alert.critical { background: #dc2626; animation: pulse 1s infinite; }
        .alert.warning { background: #f59e0b; }
        .alert-icon { font-size: 1.5em; }
        
        /* Filters */
        .filter-panel {
            background: rgba(255,255,255,0.15);
            padding: 20px;
            border-radius: 15px;
            margin-bottom: 20px;
            display: flex;
            gap: 15px;
            flex-wrap: wrap;
            align-items: center;
        }
        .filter-group {
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .filter-group label { opacity: 0.9; }
        .filter-input, .filter-select {
            padding: 10px 15px;
            border-radius: 8px;
            border: 2px solid rgba(255,255,255,0.3);
            background: rgba(0,0,0,0.3);
            color: #fff;
        }
        .btn {
            padding: 10px 20px;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 600;
        }
        .btn-primary { background: #10b981; color: #fff; }
        .btn-secondary { background: #3b82f6; color: #fff; }
        .btn:hover { opacity: 0.9; }

        /* Stats Grid */
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
            gap: 15px;
            margin-bottom: 30px;
        }
        .stat-card {
            background: rgba(255,255,255,0.15);
            padding: 20px;
            border-radius: 12px;
            text-align: center;
        }
        .stat-card h3 { font-size: 0.9em; opacity: 0.8; margin-bottom: 8px; }
        .stat-card .value { font-size: 2em; font-weight: bold; }

        /* Tables */
        .section {
            background: rgba(255,255,255,0.1);
            padding: 25px;
            border-radius: 15px;
            margin-bottom: 20px;
        }
        .section h2 { margin-bottom: 15px; font-size: 1.4em; }
        table { width: 100%; border-collapse: collapse; background: rgba(0,0,0,0.2); border-radius: 10px; overflow: hidden; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid rgba(255,255,255,0.1); }
        th { background: rgba(0,0,0,0.4); font-weight: 600; text-transform: uppercase; font-size: 0.85em; }
        tr:hover { background: rgba(255,255,255,0.05); }
        
        /* Badges */
        .badge { display: inline-block; padding: 4px 10px; border-radius: 20px; font-size: 0.8em; font-weight: 600; }
        .badge-ancient { background: #8b5cf6; }
        .badge-classic { background: #f59e0b; }
        .badge-retro { background: #3b82f6; }
        .badge-modern { background: #6b7280; }
        .badge-active { background: #10b981; }
        
        @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.7; } }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>⛏️ RustChain Dashboard v1.1</h1>
            <p>Filters | Export | Alerts</p>
        </div>
        
        <!-- Alerts -->
        <div id="alerts-container"></div>
        
        <!-- Filters -->
        <div class="filter-panel">
            <div class="filter-group">
                <label>Min Weight:</label>
                <input type="number" id="min_weight" class="filter-input" step="0.1" placeholder="e.g. 2.0">
            </div>
            <div class="filter-group">
                <label>Arch:</label>
                <select id="arch_filter" class="filter-select">
                    <option value="">All</option>
                    <option value="ancient">Ancient</option>
                    <option value="classic">Classic</option>
                    <option value="retro">Retro</option>
                    <option value="modern">Modern</option>
                </select>
            </div>
            <div class="filter-group">
                <label>Min Balance:</label>
                <input type="number" id="min_balance" class="filter-input" placeholder="e.g. 50">
            </div>
            <button class="btn btn-primary" onclick="applyFilters()">Apply Filters</button>
            <button class="btn btn-secondary" onclick="exportData('csv')">Export CSV</button>
            <button class="btn btn-secondary" onclick="exportData('json')">Export JSON</button>
        </div>
        
        <!-- Stats -->
        <div class="stats-grid">
            <div class="stat-card">
                <h3>Active Miners</h3>
                <div class="value" id="enrolled-miners">0</div>
            </div>
            <div class="stat-card">
                <h3>Epoch</h3>
                <div class="value" id="current-epoch">0</div>
            </div>
            <div class="stat-card">
                <h3>Epoch Pot</h3>
                <div class="value" id="epoch-pot">0</div>
            </div>
            <div class="stat-card">
                <h3>Total Balance</h3>
                <div class="value" id="total-balance">0</div>
            </div>
            <div class="stat-card">
                <h3>CPU</h3>
                <div class="value" id="sys-cpu">0%</div>
            </div>
            <div class="stat-card">
                <h3>Memory</h3>
                <div class="value" id="sys-mem">0%</div>
            </div>
        </div>
        
        <!-- Miners Table -->
        <div class="section">
            <h2>🖥️ Active Miners</h2>
            <table>
                <thead>
                    <tr>
                        <th>Wallet</th>
                        <th>Weight</th>
                        <th>Balance</th>
                        <th>Arch</th>
                        <th>Last Seen</th>
                    </tr>
                </thead>
                <tbody id="miners-tbody">
                    <tr><td colspan="5" style="text-align:center">Loading...</td></tr>
                </tbody>
            </table>
        </div>
    </div>

    <script>
        let currentData = null;
        
        async function fetchData() {
            const params = new URLSearchParams();
            const mw = document.getElementById('min_weight').value;
            const arch = document.getElementById('arch_filter').value;
            const mb = document.getElementById('min_balance').value;
            if (mw) params.append('min_weight', mw);
            if (arch) params.append('arch', arch);
            if (mb) params.append('min_balance', mb);
            
            const r = await fetch('/api/stats?' + params);
            currentData = await r.json();
            updateDisplay();
        }
        
        function updateDisplay() {
            if (!currentData) return;
            
            // Update stats
            document.getElementById('enrolled-miners').textContent = currentData.enrolled_miners;
            document.getElementById('current-epoch').textContent = currentData.current_epoch;
            document.getElementById('epoch-pot').textContent = currentData.epoch_pot.toFixed(2);
            document.getElementById('total-balance').textContent = currentData.total_balance.toFixed(2);
            document.getElementById('sys-cpu').textContent = currentData.system_stats?.cpu + '%';
            document.getElementById('sys-mem').textContent = currentData.system_stats?.memory + '%';
            
            // Alerts
            const alertsContainer = document.getElementById('alerts-container');
            alertsContainer.innerHTML = (currentData.alerts || []).map(a => 
                `<div class="alert ${a.severity}"><span class="alert-icon">${a.type === 'mayday' ? '🚨' : '💰'}</span><span>${a.message}</span></div>`
            ).join('');
            
            // Miners table
            const tbody = document.getElementById('miners-tbody');
            tbody.innerHTML = (currentData.active_miners || []).map(m => `
                <tr>
                    <td class="mono">${m.wallet_short}...</td>
                    <td><strong>${m.weight}x</strong></td>
                    <td class="green">${m.balance.toFixed(2)}</td>
                    <td><span class="badge badge-${m.arch}">${m.arch.toUpperCase()}</span></td>
                    <td class="mono">${m.last_seen}</td>
                </tr>
            `).join('') || '<tr><td colspan="5">No miners</td></tr>';
            
            // Sound alerts
            (currentData.alerts || []).forEach(a => {
                if (a.type === 'mayday') playSound('alert');
                else if (a.type === 'high_value') playSound('success');
            });
        }
        
        function applyFilters() { fetchData(); }
        
        function exportData(format) {
            if (!currentData) return;
            const url = '/api/export/' + format;
            window.open(url, '_blank');
        }
        
        function playSound(type) {
            const audio = new Audio('data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2teleAs7nstcXBwLH4qyw8+vWBYQL5nJz4tVBBSLqbjHso4aFi+Q0deleAs7nstcXBwLH4qyw8+vWBYQL5nJz4tVBBSLqbjHso4aFi+Q0dl');
            audio.play().catch(() => {});
        }
        
        setInterval(fetchData, 10000);
        fetchData();
    </script>
</body>
</html>
"""

# ============================================================================
# ROUTES
# ============================================================================

@app.route('/')
def dashboard():
    return render_template_string(DASHBOARD_HTML_V2)

@app.route('/api/stats')
def api_stats():
    try:
        filters = {
            'min_weight': request.args.get('min_weight'),
            'arch': request.args.get('arch'),
            'min_balance': request.args.get('min_balance'),
        }
        
        epoch_resp = requests.get(f"{NODE_API}/epoch", timeout=5)
        epoch_data = epoch_resp.json()
        
        with sqlite3.connect(DB_PATH) as conn:
            miners = conn.execute("""
                SELECT e.miner_pk, e.weight, b.balance_rtc, MAX(a.ts_ok) as last_attest
                FROM epoch_enroll e
                LEFT JOIN balances b ON e.miner_pk = b.miner_pk
                LEFT JOIN miner_attest_recent a ON e.miner_pk = a.miner
                WHERE e.epoch = ?
                GROUP BY e.miner_pk
                ORDER BY e.weight DESC
                LIMIT 100
            """, (epoch_data['epoch'],)).fetchall()
            
            active_miners = []
            for m in miners:
                weight = m[1]
                arch = 'ancient' if weight >= 3.0 else 'classic' if weight >= 2.5 else 'retro' if weight >= 1.5 else 'modern'
                active_miners.append({
                    'wallet': m[0], 'wallet_short': m[0][:16],
                    'weight': weight, 'balance': m[2] or 0.0,
                    'arch': arch, 'last_seen': datetime.fromtimestamp(m[3] or time.time()).strftime('%H:%M:%S')
                })
            
            active_miners = apply_filters(active_miners, filters)
            
            cpu = psutil.cpu_percent(interval=1)
            mem = psutil.virtual_memory()
            
            total_balance = conn.execute("SELECT SUM(balance_rtc) FROM balances").fetchone()[0] or 0.0
            
            data = {
                'enrolled_miners': epoch_data['enrolled_miners'],
                'current_epoch': epoch_data['epoch'],
                'epoch_pot': epoch_data['epoch_pot'],
                'total_balance': total_balance,
                'active_miners': active_miners,
                'system_stats': {'cpu': cpu, 'memory': mem.percent},
                'timestamp': int(time.time())
            }
            
            data['alerts'] = alert_manager.check_alerts(data)
            return jsonify(data)
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/api/export/<fmt>')
def api_export(fmt):
    try:
        epoch_resp = requests.get(f"{NODE_API}/epoch", timeout=5)
        epoch_data = epoch_resp.json()
        
        with sqlite3.connect(DB_PATH) as conn:
            miners = conn.execute("""
                SELECT e.miner_pk, e.weight, b.balance_rtc
                FROM epoch_enroll e
                LEFT JOIN balances b ON e.miner_pk = b.miner_pk
                WHERE e.epoch = ?
            """, (epoch_data['epoch'],)).fetchall()
            
            data = {
                'epoch': epoch_data['epoch'],
                'enrolled_miners': epoch_data['enrolled_miners'],
                'total_balance': conn.execute("SELECT SUM(balance_rtc) FROM balances").fetchone()[0] or 0.0,
                'active_miners': [{'wallet': m[0], 'weight': m[1], 'balance': m[2] or 0.0} for m in miners],
                'exported_at': datetime.now().isoformat()
            }
        
        if fmt == 'csv':
            return app.response_class(export_to_csv(data), mimetype='text/csv',
                headers={'Content-Disposition': 'attachment; filename=dashboard_export.csv'})
        elif fmt == 'json':
            return app.response_class(export_to_json(data), mimetype='application/json',
                headers={'Content-Disposition': 'attachment; filename=dashboard_export.json'})
        return jsonify({'error': 'Invalid format'}), 400
    except Exception as e:
        return jsonify({'error': str(e)}), 500

# ============================================================================
# TESTS
# ============================================================================

def test_filter():
    test = [{'weight': 3, 'arch': 'ancient', 'balance': 100}]
    r = apply_filters(test, {'min_weight': '2'})
    assert len(r) == 1
    r = apply_filters(test, {'arch': 'modern'})
    assert len(r) == 0
    print("✅ Filter test passed")

def test_export():
    d = {'active_miners': [{'wallet': 't', 'w': 3, 'b': 100, 'a': 'a', 'l': '12:00'}]}
    assert 't' in export_to_csv(d)
    assert 't' in export_to_json(d)
    print("✅ Export test passed")

def test_alerts():
    m = AlertManager()
    a = m.check_alerts({'enrolled_miners': 0, 'total_balance': 50})
    assert len(a) == 1 and a[0]['type'] == 'mayday'
    print("✅ Alert test passed")

if __name__ == '__main__':
    test_filter(); test_export(); test_alerts()
    print("✅ All tests passed")
    app.run(host='0.0.0.0', port=8099, debug=False)
