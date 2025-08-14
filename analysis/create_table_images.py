#!/usr/bin/env python3
"""
Create table images for Medium article
"""

import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch
import numpy as np

# Set style
plt.style.use('seaborn-v0_8-darkgrid')

def create_performance_table():
    """Create the Fair Concurrent Benchmarks table"""
    
    fig, ax = plt.subplots(figsize=(14, 4))
    ax.axis('tight')
    ax.axis('off')
    
    # Table data from latest benchmarks - with REAL parallelism metrics
    headers = ['Implementation', 'Throughput (ops/sec)', 'CPU Efficiency', 'Memory (MB)']
    data = [
        ['Python (Claude)', '43,798', '1.0% (GIL)', '245'],
        ['Java (Qwen-30B)', '258,793', '25.9%', '156'],
        ['Rust (Qwen-30B)', '72,675', '7.3%', '87'],
        ['Rust (Qwen-235B)', '227,179', '22.7%', '82'],
        ['Rust (Qwen-435B)', '289,234', '28.9%', '78'],
        ['Rust (GLM-4.5)', 'Failed', '-', '-']
    ]
    
    # Create table positioned at top
    table = ax.table(cellText=data,
                    colLabels=headers,
                    cellLoc='center',
                    loc='upper center',
                    colWidths=[0.35, 0.25, 0.2, 0.2])
    
    # Style the table
    table.auto_set_font_size(False)
    table.set_fontsize(12)
    table.scale(1, 1.5)
    
    # Color coding
    colors = {
        'header': '#667eea',
        'python': '#3776ab',
        'java': '#007396', 
        'rust': '#ce422b',
        'glm': '#888888'
    }
    
    # Style header
    for i in range(len(headers)):
        table[(0, i)].set_facecolor(colors['header'])
        table[(0, i)].set_text_props(weight='bold', color='white')
        table[(0, i)].set_height(0.08)
    
    # Style rows
    table[(1, 0)].set_facecolor(colors['python'] + '30')  # Python
    table[(2, 0)].set_facecolor(colors['java'] + '30')    # Java
    for i in range(3, 6):  # Rust implementations
        table[(i, 0)].set_facecolor(colors['rust'] + '30')
    table[(6, 0)].set_facecolor(colors['glm'] + '30')     # GLM
    
    # Highlight best values
    table[(5, 1)].set_text_props(weight='bold', color='green')  # Best throughput
    table[(5, 3)].set_text_props(weight='bold', color='green')  # Best memory
    
    # Set proper row heights for readability
    for i in range(len(headers)):
        table[(0, i)].set_height(0.08)  # Header row
    for i in range(1, 7):
        for j in range(len(headers)):
            table[(i, j)].set_height(0.08)  # Data rows
    
    # Remove title - just save the table
    # Get the table's bounding box and crop to it
    renderer = fig.canvas.get_renderer()
    bbox = table.get_window_extent(renderer)
    bbox_inches = bbox.transformed(fig.dpi_scale_trans.inverted())
    plt.savefig('analysis/performance_table.png', dpi=150, bbox_inches=bbox_inches, 
                facecolor='white', edgecolor='none', pad_inches=0)
    plt.close()
    print("Created: performance_table.png")

def create_single_thread_table():
    """Create the Single-Thread Performance table"""
    
    fig, ax = plt.subplots(figsize=(14, 3))
    ax.axis('tight')
    ax.axis('off')
    
    headers = ['Operation', 'Python', 'Java', 'Rust-30B', 'Rust-235B', 'Rust-435B']
    data = [
        ['PUT', '12.5 μs', '4.2 μs', '2.1 μs', '1.8 μs', '0.9 μs'],
        ['GET (hit)', '8.3 μs', '2.1 μs', '0.7 μs', '0.6 μs', '0.3 μs'],
        ['GET (miss)', '8.3 μs', '2.1 μs', '0.7 μs', '0.6 μs', '0.3 μs']
    ]
    
    table = ax.table(cellText=data,
                    colLabels=headers,
                    cellLoc='center',
                    loc='upper center',
                    colWidths=[0.2, 0.16, 0.16, 0.16, 0.16, 0.16])
    
    table.auto_set_font_size(False)
    table.set_fontsize(12)
    table.scale(1, 1.5)
    
    # Style header
    for i in range(len(headers)):
        table[(0, i)].set_facecolor('#667eea')
        table[(0, i)].set_text_props(weight='bold', color='white')
    
    # Highlight best values (Rust-435B column)
    for i in range(1, 4):
        table[(i, 5)].set_text_props(weight='bold', color='green')
        table[(i, 5)].set_facecolor('#90EE9030')
    
    # Set proper row heights for readability
    for i in range(len(headers)):
        table[(0, i)].set_height(0.1)  # Header row
    for i in range(1, 4):
        for j in range(len(headers)):
            table[(i, j)].set_height(0.1)  # Data rows
    
    # Remove title - just save the table
    # Get the table's bounding box and crop to it
    renderer = fig.canvas.get_renderer()
    bbox = table.get_window_extent(renderer)
    bbox_inches = bbox.transformed(fig.dpi_scale_trans.inverted())
    plt.savefig('analysis/single_thread_table.png', dpi=150, bbox_inches=bbox_inches,
                facecolor='white', edgecolor='none', pad_inches=0)
    plt.close()
    print("Created: single_thread_table.png")

def create_architecture_table():
    """Create the Data Structure Choices table"""
    
    fig, ax = plt.subplots(figsize=(14, 4))
    ax.axis('tight')
    ax.axis('off')
    
    headers = ['Model', 'Primary Storage', 'LRU Tracking', 'Complexity']
    data = [
        ['Claude', 'OrderedDict', 'Built-in', 'O(1)'],
        ['Qwen-30B Java', 'ConcurrentHashMap', 'PriorityQueue', 'O(n)'],
        ['Qwen-30B Rust', 'HashMap', 'VecDeque', 'O(n)'],
        ['Qwen-235B', 'HashMap', 'Custom LinkedList', 'O(1)'],
        ['Qwen-435B', 'DashMap', 'LinkedList', 'O(1)'],
        ['GLM-4.5', 'HashMap', 'LinkedList', 'O(1)*']
    ]
    
    table = ax.table(cellText=data,
                    colLabels=headers,
                    cellLoc='center',
                    loc='upper center',
                    colWidths=[0.25, 0.3, 0.3, 0.15])
    
    table.auto_set_font_size(False)
    table.set_fontsize(12)
    table.scale(1, 1.5)
    
    # Style header - use same blue as other tables
    for i in range(len(headers)):
        table[(0, i)].set_facecolor('#667eea')
        table[(0, i)].set_text_props(weight='bold', color='white')
    
    # Highlight O(1) complexity cells
    for i in [1, 4, 5, 6]:
        if 'O(1)' in data[i-1][3]:
            table[(i, 3)].set_facecolor('#90EE9030')
            table[(i, 3)].set_text_props(weight='bold')
    
    # Set proper row heights for readability
    for i in range(len(headers)):
        table[(0, i)].set_height(0.08)  # Header row
    for i in range(1, 7):
        for j in range(len(headers)):
            table[(i, j)].set_height(0.08)  # Data rows
    
    # Remove title - just save the table
    # Add footnote as text annotation
    ax.text(0.5, -0.02, '*GLM-4.5 had correct algorithmic complexity but failed to compile',
            ha='center', fontsize=10, style='italic', transform=ax.transAxes)
    
    # Get the table's bounding box and crop to it (include footnote)
    renderer = fig.canvas.get_renderer()
    bbox = table.get_window_extent(renderer)
    bbox_inches = bbox.transformed(fig.dpi_scale_trans.inverted())
    # Extend bbox slightly down for footnote
    bbox_inches.y0 -= 0.2
    plt.savefig('analysis/architecture_table.png', dpi=150, bbox_inches=bbox_inches,
                facecolor='white', edgecolor='none', pad_inches=0)
    plt.close()
    print("Created: architecture_table.png")

def create_final_rankings_table():
    """Create the Final Rankings table"""
    
    fig, ax = plt.subplots(figsize=(14, 4))
    ax.axis('tight')
    ax.axis('off')
    
    headers = ['Rank', 'Model', 'Score', 'Key Strength', 'Fatal Flaw']
    data = [
        ['1', 'Qwen-435B', '94/100', 'DashMap sharding', 'Complex code'],
        ['2', 'Qwen-235B', '91/100', 'Perfect O(1) LRU', 'Deadlock risk'],
        ['3', 'GLM-4.5', '89/100*', 'Most innovative', "Doesn't compile"],
        ['4', 'Qwen-30B Rust', '85/100', 'Solid, safe', 'O(n) operations'],
        ['5', 'Qwen-30B Java', '82/100', 'Enterprise-ready', 'GC pauses'],
        ['6', 'Claude', '78/100', 'Cleanest code', 'GIL bottleneck']
    ]
    
    table = ax.table(cellText=data,
                    colLabels=headers,
                    cellLoc='center',
                    loc='upper center',
                    colWidths=[0.1, 0.25, 0.15, 0.25, 0.25])
    
    table.auto_set_font_size(False)
    table.set_fontsize(12)
    table.scale(1, 1.5)
    
    # Style header
    for i in range(len(headers)):
        table[(0, i)].set_facecolor('#667eea')
        table[(0, i)].set_text_props(weight='bold', color='white')
    
    # Color code ranks
    rank_colors = ['#FFD700', '#C0C0C0', '#CD7F32', '#E5E5E5', '#E5E5E5', '#E5E5E5']
    for i in range(1, 7):
        table[(i, 0)].set_facecolor(rank_colors[i-1] + '60')
        table[(i, 1)].set_text_props(weight='bold')
        
        # Color score cells based on value
        score = int(data[i-1][2].split('/')[0].replace('*', ''))
        if score >= 90:
            table[(i, 2)].set_facecolor('#90EE9030')
        elif score >= 85:
            table[(i, 2)].set_facecolor('#FFEB3B30')
    
    # Set proper row heights for readability
    for i in range(len(headers)):
        table[(0, i)].set_height(0.08)  # Header row
    for i in range(1, 7):
        for j in range(len(headers)):
            table[(i, j)].set_height(0.08)  # Data rows
    
    # Remove title - just save the table
    # Get the table's bounding box and crop to it
    renderer = fig.canvas.get_renderer()
    bbox = table.get_window_extent(renderer)
    bbox_inches = bbox.transformed(fig.dpi_scale_trans.inverted())
    plt.savefig('analysis/final_rankings_table.png', dpi=150, bbox_inches=bbox_inches,
                facecolor='white', edgecolor='none', pad_inches=0)
    plt.close()
    print("Created: final_rankings_table.png")

def create_io_simulation_table():
    """Create I/O Simulation Results table"""
    
    fig, ax = plt.subplots(figsize=(12, 3))
    ax.axis('tight')
    ax.axis('off')
    
    headers = ['Implementation', 'Speedup', 'Total Ops', 'Notes']
    data = [
        ['Python', '176x', '147,850', 'Threading helps I/O'],
        ['Java', '184x', '153,868', 'Thread pool efficient'],
        ['Rust (all)', '175x', '145,000', 'Similar I/O benefits']
    ]
    
    table = ax.table(cellText=data,
                    colLabels=headers,
                    cellLoc='center',
                    loc='upper center',
                    colWidths=[0.25, 0.2, 0.2, 0.35])
    
    table.auto_set_font_size(False)
    table.set_fontsize(12)
    table.scale(1, 1.5)
    
    # Style header
    for i in range(len(headers)):
        table[(0, i)].set_facecolor('#667eea')
        table[(0, i)].set_text_props(weight='bold', color='white')
    
    # Highlight high speedup values
    for i in range(1, 4):
        table[(i, 1)].set_facecolor('#90EE9030')
        table[(i, 1)].set_text_props(weight='bold')
    
    # Set proper row heights for readability
    for i in range(len(headers)):
        table[(0, i)].set_height(0.1)  # Header row
    for i in range(1, 4):
        for j in range(len(headers)):
            table[(i, j)].set_height(0.1)  # Data rows
    
    # Remove title - just save the table
    # Get the table's bounding box and crop to it
    renderer = fig.canvas.get_renderer()
    bbox = table.get_window_extent(renderer)
    bbox_inches = bbox.transformed(fig.dpi_scale_trans.inverted())
    plt.savefig('analysis/io_simulation_table.png', dpi=150, bbox_inches=bbox_inches,
                facecolor='white', edgecolor='none', pad_inches=0)
    plt.close()
    print("Created: io_simulation_table.png")

if __name__ == "__main__":
    print("Creating table images for Medium article...")
    create_performance_table()
    create_single_thread_table()
    create_architecture_table()
    create_final_rankings_table()
    create_io_simulation_table()
    print("\nAll table images created successfully!")
    print("Files created in the analysis directory:")
    print("  - performance_table.png")
    print("  - single_thread_table.png")
    print("  - architecture_table.png")
    print("  - final_rankings_table.png")
    print("  - io_simulation_table.png")