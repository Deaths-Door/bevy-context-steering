import numpy as np
import plotly.graph_objects as go
import math
def generate_sphere(sample_size):
    points = []
    golden_angle = np.pi * (np.sqrt(5.0) - 1.0)
    for i in range(sample_size):
        y = 1.0 - (i / (sample_size - 1.0)) * 2.0
        radius = np.sqrt(max(0, 1.0 - y * y))
        theta = golden_angle * i
        x = radius * np.cos(theta)
        z = radius * np.sin(theta)
        points.append([x, y, z])
    return np.array(points)


# --- Configuration ---
N = 128
SELECTED_INDEX = 80

data = generate_sphere(N)
target = data[SELECTED_INDEX]

# --- The Threshold Logic ---
similarities = np.dot(data, target)
THRESHOLD = math.cos(1.25 *math.sqrt( 4 * math.pi) / math.sqrt(N))
neighbor_indices = np.where((similarities > THRESHOLD) & (similarities < 0.9999))[0]

# --- Visualise ---
fig = go.Figure()
fig.add_trace(go.Scatter3d(x=data[:, 0], y=data[:, 1], z=data[:, 2], mode='markers', marker=dict(size=2, color='blue', opacity=0.2)))
fig.add_trace(go.Scatter3d(x=[target[0]], y=[target[1]], z=[target[2]], mode='markers', marker=dict(size=10, color='lime')))

for n_idx in neighbor_indices:
    n_pt = data[n_idx]
    fig.add_trace(go.Scatter3d(x=[target[0], n_pt[0]], y=[target[1], n_pt[1]], z=[target[2], n_pt[2]], mode='lines+markers', line=dict(color='red', width=5)))

fig.update_layout(scene=dict(aspectmode='cube'))
fig.show()