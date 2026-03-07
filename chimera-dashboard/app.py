import streamlit as st
import plotly.graph_objects as go

st.title("⚡ ChimeraOS Dashboard")

if st.session_state.connected:
    stats = st.session_state.client.get_global_stats()
    cols = st.columns(5)
    cols[0].metric("Total Hashrate", f"{stats['hashrate'] / 1e12:.2f} TH/s")
    cols[1].metric("Power Draw", f"{stats['power']:.1f} kW")