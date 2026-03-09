import streamlit as st
import plotly.graph_objects as go

st.title("ChimeraOS Dashboard")

if st.session_state.connected:
    stats = st.session_state.client.get_global_stats()
    cols = st.columns(5)
    cols[0].metric("Total Hashrate", f"{stats['hashrate'] / 1e12:.2f} TH/s")
    cols[1].metric("Power Draw", f"{stats['power']:.1f} kW")


# chimera-dashboard/app.py
pareto_data = client.get_pareto_front()
fig = go.Figure(data=go.Scatter3d(
    x=[p.objectives[0] for p in pareto_data.points],
    y=[p.objectives[1] for p in pareto_data.points],
    z=[p.objectives[2] for p in pareto_data.points],
    mode='markers'
))
st.plotly_chart(fig)