import subprocess
import time
import json
import sys
from pathlib import Path

eval_dir = Path(__file__).resolve().parent

scenarios = [
    {"name": "No Tool Access (Baseline)", "mode": "no-tool"},
    {"name": "tq CLI (No Skill)", "mode": "tq-cli-no-skill"},
    {"name": "tq CLI (On-Demand Skill)", "mode": "tq-cli"},
    {"name": "Teradata MCP Server CE", "mode": "mcp"}
]

results = []

for s in scenarios:
    print(f"\n==================================================")
    print(f"Running Scenario: {s['name']} (Mode: {s['mode']})")
    print(f"==================================================")
    
    start_time = time.time()
    
    # Spawn MCP server in the background if running the MCP scenario
    server_proc = None
    if s["mode"] == "mcp":
        import os
        user = os.environ.get("TERADATA_USER", "demo_user")
        password = os.environ.get("TERADATA_PASSWORD", "demo_user")
        host = os.environ.get("TERADATA_HOST", "trial-vikzqtnd0db0nglk.env.trial.teradata.com")
        db = os.environ.get("EVALS_DATABASE", "demo_user")
        database_uri = f"teradata://{user}:{password}@{host}:1025/{db}"
        
        env = os.environ.copy()
        env["DATABASE_URI"] = database_uri
        
        server_dir = Path("/Users/remi.turpaud/Code/genAI/teradata-mcp-server")
        python_exec = server_dir / ".venv/bin/python"
        
        print("Starting Teradata MCP Server under the 'base' profile...")
        server_proc = subprocess.Popen(
            [
                str(python_exec), "-m", "teradata_mcp_server",
                "--profile", "base",
                "--config_dir", str(server_dir)
            ],
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
        time.sleep(5)  # Let the server boot up
    
    # Run the evaluation command
    cmd = [
        ".venv/bin/python", "run_evals.py",
        "--module", "base",
        "--type", "happy_path",
        "--mode", s["mode"],
        "--provider", "gemini",
        "--skip-preflight"
    ]
    
    proc = subprocess.run(cmd, cwd=str(eval_dir))
    duration = time.time() - start_time
    
    # Terminate the MCP server if it was spawned
    if server_proc:
        print("Stopping Teradata MCP Server...")
        server_proc.terminate()
        server_proc.wait()
    
    # Load results from latest_summary.json
    latest_json_path = eval_dir / "results/latest_summary.json"
    if latest_json_path.exists():
        with open(latest_json_path, "r") as f:
            summary = json.load(f)
            
        passed = summary.get("passed", 0)
        total = summary.get("total", 0)
        
        # Calculate tokens/cost aggregates from the cases list
        input_tokens = sum(c.get("input_tokens", 0) for c in summary.get("cases", []))
        output_tokens = sum(c.get("output_tokens", 0) for c in summary.get("cases", []))
        total_tokens = sum(c.get("total_tokens", 0) for c in summary.get("cases", []))
        cost = sum(c.get("cost", 0.0) for c in summary.get("cases", []))
        
        results.append({
            "name": s["name"],
            "mode": s["mode"],
            "pass_rate": f"{passed}/{total}",
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "total_tokens": total_tokens,
            "cost": cost,
            "duration": duration
        })
        print(f"\n>>> Scenario {s['name']} Finished: {passed}/{total} passed in {duration:.1f}s. Cost: ${cost:.5f}")
    else:
        print(f"Error: {latest_json_path} not found.")

# Write consolidated report
report_path = eval_dir / "results/comparison_report.md"
with open(report_path, "w") as f:
    f.write("# Teradata Agent Interface Comparison Report\n\n")
    f.write("This report compares the effectiveness, efficiency, and costs of different agent interfaces for Teradata database exploration.\n\n")
    f.write("## Overview Metrics\n\n")
    f.write("| Interface / Scenario | Pass Rate | Input Tokens | Output Tokens | Total Tokens | Cost (USD) | Duration (s) | Speed (tokens/s) |\n")
    f.write("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n")
    for r in results:
        speed = r["total_tokens"] / r["duration"] if r["duration"] > 0 else 0
        f.write(f"| {r['name']} (`{r['mode']}`) | {r['pass_rate']} | {r['input_tokens']:,} | {r['output_tokens']:,} | {r['total_tokens']:,} | ${r['cost']:.5f} | {r['duration']:.1f}s | {speed:.1f}/s |\n")
    
    f.write("\n## Key Findings\n\n")
    f.write("1. **Token Efficiency**: Measure context savings from using the on-demand skill model vs full context load.\n")
    f.write("2. **Cost Effectiveness**: Compare tool execution via MCP functions vs tq CLI command line execution.\n")
    f.write("3. **Operational Speed**: Observe total latency to fulfill all tasks.\n")

print(f"\nConsolidated comparison report successfully written to: {report_path}")
