import matplotlib.pyplot as plt
import numpy as np


threads = [1, 2, 5, 10, 100, 1000, 10000]
thread_creation = [0.000401, 0.000381, 0.000383, 0.000318, 0.000319, 0.000401, 0.000578]
document_loading = [0.950944, 1.020256, 1.282666, 1.531044, 1.897386, 4.041009, 6.032099]
index_update = [116.594275, 60.42831, 31.239057, 18.827697, 23.076418, 16.464164, 12.517071]

plt.figure(figsize=(12, 6))
width = 0.2

x = np.arange(len(threads))

plt.bar(x - width, thread_creation, width, label="Thread Creation (s)", color="blue")
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
