import matplotlib.pyplot as plt
import numpy as np


threads = [1, 2, 5, 10, 100, 1000]
document_loading = [0.119457, 0.123407, 0.133936, 0.148583, 0.178151, 0.66946]
index_update = [10.056386, 5.201664, 2.203428, 1.516104, 1.449091, 1.133619]

plt.figure(figsize=(12, 6))
width = 0.2

x = np.arange(len(threads))

plt.bar(x, document_loading, width, label="Document Loading (s)", color="green")
plt.bar(x + width, index_update, width, label="Index Update (s)", color="orange")

# Labels and legend
plt.xticks(x, threads)
plt.xlabel("Number of Threads")
plt.ylabel("Time (seconds)")
plt.title("Performance Analysis for Different Thread Counts")
plt.legend()

plt.tight_layout()
plt.savefig("output.png")
