import json
import matplotlib.pyplot as plt
import plotly.graph_objects as go
from plotly.subplots import make_subplots
import pandas as pd
import numpy as np

def load_data(filename):
    with open(filename, 'r', encoding='utf-8-sig') as f:
        return json.load(f)

# Load data
sim_data = load_data('dynamics_data_rigorous.json')
df_dyn_pressure = pd.DataFrame(sim_data['dynamics'])
df_dyn_bench = pd.DataFrame(sim_data['benchmarks'])
metadata = sim_data['metadata']

# --- 1. Matplotlib Static Pressure Diagram ---
plt.style.use('dark_background')
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

# Plot Entropy vs Temperature
ax1.plot(df_dyn_pressure['tau'], df_dyn_pressure['soft_entropy'], label='Soft Entropy (Proposal)', color='#58a6ff')
ax1.plot(df_dyn_pressure['tau'], df_dyn_pressure['exec_entropy'], label='Exec Entropy (Governance)', color='#3fb950')
ax1.set_title('Governance Entropy Profile')
ax1.set_xlabel('Temperature (Tau)')
ax1.set_ylabel('Entropy (S)')
ax1.legend()
ax1.grid(alpha=0.2)

# Plot Rejection Pressure
ax2.fill_between(df_dyn_pressure['tau'], df_dyn_pressure['rejection_pressure'], color='#f85149', alpha=0.3)
ax2.plot(df_dyn_pressure['tau'], df_dyn_pressure['rejection_pressure'], color='#f85149', label='Rejection Pressure')
ax2.set_title('Verifier Rejection Pressure (Delta S)')
ax2.set_xlabel('Temperature (Tau)')
ax2.set_ylabel('Pressure (S_soft - S_exec)')
ax2.legend()
ax2.grid(alpha=0.2)

plt.tight_layout()
plt.savefig('coh_rigorous_pressure.png', dpi=150)
print("Saved coh_rigorous_pressure.png")

# --- 2. Plotly Interactive Dashboard ---
fig_plotly = make_subplots(
    rows=2, cols=2,
    subplot_titles=(
        "Isolated Verifier Latency (Non-Abelian)", 
        "Verified Stack Memory (Bytes)",
        "Rigorous SU(2) Holonomy Trace",
        "Rejection Pressure Sweep"
    )
)

# Latency
fig_plotly.add_trace(
    go.Scatter(x=df_dyn_bench['steps'], y=df_dyn_bench['latency_us'], name='Latency', line=dict(color='#d29922')),
    row=1, col=1
)

# Memory (Dynamic from metadata)
mem_labels = ['CohBit', 'CohAtom']
mem_values = [metadata['cohbit_stack_bytes'], metadata['cohatom_stack_bytes']]
fig_plotly.add_trace(
    go.Bar(x=mem_labels, y=mem_values, name='Memory', marker_color='#bc8cff'),
    row=1, col=2
)

# Holonomy (Rigorous SU(2))
fig_plotly.add_trace(
    go.Scatter(x=df_dyn_bench['steps'], y=df_dyn_bench['result_holonomy'], name='Tr(W)', line=dict(color='#3fb950')),
    row=2, col=1
)

# Pressure
fig_plotly.add_trace(
    go.Scatter(x=df_dyn_pressure['tau'], y=df_dyn_pressure['rejection_pressure'], name='Pressure', fill='tozeroy', line=dict(color='#f85149')),
    row=2, col=2
)

fig_plotly.update_layout(
    title_text=f"Rigorous Coh Analytics (CohBit: {metadata['cohbit_stack_bytes']}B, Atom: {metadata['cohatom_stack_bytes']}B)",
    template="plotly_dark",
    showlegend=False,
    height=800
)

fig_plotly.write_html('coh_rigorous_analytics.html')
print("Saved coh_rigorous_analytics.html")
