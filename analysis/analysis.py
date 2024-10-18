import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
import sqlalchemy as sql

# Create a database connection
engine = sql.create_engine('postgresql://localhost:5432/robopoker')
query =  """
    WITH unique_abstraction AS (
        SELECT DISTINCT abs
        FROM turn_abs
    ),
    self_join_abstraction AS (
        SELECT 
            a.abs as abs1,
            b.abs as abs2,
            (a.abs # b.abs)::numeric as xor_result
        FROM unique_abstraction a
        CROSS JOIN unique_abstraction b
        WHERE a.abs > b.abs
    )
    SELECT 
        c.abs1,
        c.abs2,
        COALESCE(m.dst, 0) as dst
    FROM self_join_abstraction c
    LEFT JOIN turn_met m ON m.xab = c.xor_result::bigint
"""


# Read the query directly into a pandas DataFrame
df = pd.read_sql(query, engine)

df['x'] = df['abs1']
df['y'] = df['abs2']

# After loading the data
abstractions_hex = sorted(set(df['x'].unique()) | set(df['y'].unique()))
complete_index = pd.MultiIndex.from_product([abstractions_hex, abstractions_hex], names=['x', 'y'])
complete_df = df.set_index(['x', 'y']).reindex(complete_index).reset_index()
complete_df['dst'] = complete_df['dst'].fillna(0)  # Fill NaN with 0 or another appropriate value

# Create a pivot table with the complete dataset
pivot_df = complete_df.pivot(index='x', columns='y', values='dst')

# Create a heatmap
plt.figure(figsize=(24, 20))  # Increased figure size further
sns.heatmap(pivot_df, cmap='YlOrRd', annot=False, square=True, xticklabels=True, yticklabels=True)
plt.title('Turn Abstraction Distances')
plt.xlabel('Abstraction 2')
plt.ylabel('Abstraction 1')
plt.xticks(rotation=90, fontsize=6)
plt.yticks(rotation=0, fontsize=6)
plt.tight_layout()
plt.savefig(f'{int(pd.Timestamp.now().timestamp()) >> 8}.turn.metric.heatmap.png', dpi=300)
plt.close()

print("\nbasic statistics:")
print(df['dst'].describe())

print("\ntop 10 furthest pairs:")
print(df.nlargest(10, 'dst')[['x', 'y', 'dst']])

print("\ntop 10 closest pairs:") 
print(df.nsmallest(10, 'dst')[['x', 'y', 'dst']])

# Distribution of distances
plt.figure(figsize=(10, 6))
sns.histplot(df['dst'], kde=True)
plt.title('distribution of turn abstraction distances')
plt.xlabel('dst')
plt.savefig(f'{int(pd.Timestamp.now().timestamp()) >> 8}.turn.metric.distribution.png')
plt.close()

print("Shape of pivot_df:", pivot_df.shape)
print("Number of non-zero entries:", (pivot_df != 0).sum().sum())
