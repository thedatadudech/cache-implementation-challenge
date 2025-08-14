import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.*;
import java.io.*;
import java.nio.file.*;
import java.time.*;
import java.time.format.DateTimeFormatter;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;

/**
 * Fair Concurrent Benchmark with External Workload
 * Measures actual throughput under realistic conditions
 */
public class FairConcurrentBenchmark {
    private final int cacheSize;
    private static final Random random = new Random();
    
    public FairConcurrentBenchmark(int cacheSize) {
        this.cacheSize = cacheSize;
    }
    
    static class BenchmarkResult {
        Map<String, Object> metrics = new HashMap<>();
        
        void put(String key, Object value) {
            metrics.put(key, value);
        }
        
        Object get(String key) {
            return metrics.get(key);
        }
        
        void printResults() {
            System.out.println("\nResults:");
            for (Map.Entry<String, Object> entry : metrics.entrySet()) {
                System.out.printf("  %s: %s\n", entry.getKey(), entry.getValue());
            }
        }
    }
    
    /**
     * Producer-Consumer pattern benchmark
     */
    public BenchmarkResult benchmarkProducerConsumer(int numProducers, int numConsumers, int durationSeconds) 
            throws InterruptedException, NoSuchAlgorithmException {
        
        IntelligentCache<String, String> cache = new IntelligentCache<>(cacheSize);
        
        AtomicBoolean stopFlag = new AtomicBoolean(false);
        AtomicInteger[] producerCounts = new AtomicInteger[numProducers];
        AtomicInteger[] consumerHits = new AtomicInteger[numConsumers];
        AtomicInteger[] consumerMisses = new AtomicInteger[numConsumers];
        
        for (int i = 0; i < numProducers; i++) {
            producerCounts[i] = new AtomicInteger(0);
        }
        for (int i = 0; i < numConsumers; i++) {
            consumerHits[i] = new AtomicInteger(0);
            consumerMisses[i] = new AtomicInteger(0);
        }
        
        ExecutorService executor = Executors.newFixedThreadPool(numProducers + numConsumers);
        List<Future<?>> futures = new ArrayList<>();
        
        System.out.printf("\nRunning Producer-Consumer benchmark (%d producers, %d consumers)...\n", 
                         numProducers, numConsumers);
        System.out.printf("Duration: %d seconds\n", durationSeconds);
        
        long startTime = System.nanoTime();
        
        // Start producers
        for (int i = 0; i < numProducers; i++) {
            final int producerId = i;
            futures.add(executor.submit(() -> {
                MessageDigest md;
                try {
                    md = MessageDigest.getInstance("MD5");
                } catch (NoSuchAlgorithmException e) {
                    return;
                }
                
                int count = 0;
                while (!stopFlag.get()) {
                    String key = String.format("p%d_item_%d", producerId, count % 1000);
                    String input = String.format("%d_%d_%d", producerId, count, System.nanoTime());
                    byte[] hash = md.digest(input.getBytes());
                    String value = bytesToHex(hash);
                    
                    cache.put(key, value, random.nextInt(10) + 1, Duration.ofHours(1));
                    count++;
                    producerCounts[producerId].set(count);
                    
                    try {
                        Thread.sleep(0, 100000); // 0.1ms
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        break;
                    }
                }
            }));
        }
        
        // Start consumers
        for (int i = 0; i < numConsumers; i++) {
            final int consumerId = i;
            futures.add(executor.submit(() -> {
                MessageDigest md;
                try {
                    md = MessageDigest.getInstance("MD5");
                } catch (NoSuchAlgorithmException e) {
                    return;
                }
                
                while (!stopFlag.get()) {
                    int producerId = random.nextInt(numProducers);
                    int itemId = random.nextInt(1000);
                    String key = String.format("p%d_item_%d", producerId, itemId);
                    
                    String result = cache.get(key);
                    if (result != null) {
                        // Simulate processing
                        md.digest(result.getBytes());
                        consumerHits[consumerId].incrementAndGet();
                    } else {
                        consumerMisses[consumerId].incrementAndGet();
                    }
                    
                    try {
                        Thread.sleep(0, 100000); // 0.1ms
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        break;
                    }
                }
            }));
        }
        
        // Run for specified duration
        Thread.sleep(durationSeconds * 1000);
        stopFlag.set(true);
        
        // Wait for all to complete
        executor.shutdown();
        executor.awaitTermination(10, TimeUnit.SECONDS);
        
        long elapsed = System.nanoTime() - startTime;
        double elapsedSeconds = elapsed / 1_000_000_000.0;
        
        // Calculate statistics
        int totalPuts = 0;
        for (AtomicInteger count : producerCounts) {
            totalPuts += count.get();
        }
        
        int totalHits = 0;
        int totalMisses = 0;
        for (int i = 0; i < numConsumers; i++) {
            totalHits += consumerHits[i].get();
            totalMisses += consumerMisses[i].get();
        }
        int totalGets = totalHits + totalMisses;
        
        double hitRate = totalGets > 0 ? (double) totalHits / totalGets : 0;
        
        BenchmarkResult result = new BenchmarkResult();
        result.put("duration", elapsedSeconds);
        result.put("total_puts", totalPuts);
        result.put("total_gets", totalGets);
        result.put("total_operations", totalPuts + totalGets);
        result.put("puts_per_second", String.format("%.2f", totalPuts / elapsedSeconds));
        result.put("gets_per_second", String.format("%.2f", totalGets / elapsedSeconds));
        result.put("ops_per_second", String.format("%.2f", (totalPuts + totalGets) / elapsedSeconds));
        result.put("hit_rate", String.format("%.1f%%", hitRate * 100));
        result.put("total_hits", totalHits);
        result.put("total_misses", totalMisses);
        
        return result;
    }
    
    /**
     * Shared workload benchmark - all workers pull from common queue
     */
    public BenchmarkResult benchmarkSharedWorkload(int numWorkers, int numOperations) 
            throws InterruptedException {
        
        IntelligentCache<String, String> cache = new IntelligentCache<>(cacheSize);
        
        // Create work queue
        BlockingQueue<Object[]> workQueue = new LinkedBlockingQueue<>();
        
        // Fill with mixed operations
        for (int i = 0; i < numOperations; i++) {
            if (random.nextDouble() < 0.7) { // 70% writes
                workQueue.add(new Object[]{"PUT", String.format("key_%d", i % 1000), 
                                          String.format("value_%d", i), random.nextInt(10) + 1});
            } else { // 30% reads
                workQueue.add(new Object[]{"GET", String.format("key_%d", random.nextInt(1000))});
            }
        }
        
        List<Long> operationTimes = Collections.synchronizedList(new ArrayList<>());
        
        ExecutorService executor = Executors.newFixedThreadPool(numWorkers);
        List<Future<?>> futures = new ArrayList<>();
        
        System.out.printf("\nRunning Shared Workload benchmark (%d workers, %d operations)...\n", 
                         numWorkers, numOperations);
        
        long startTime = System.nanoTime();
        
        // Start workers
        for (int i = 0; i < numWorkers; i++) {
            futures.add(executor.submit(() -> {
                List<Long> localTimes = new ArrayList<>();
                
                while (true) {
                    Object[] task = workQueue.poll();
                    if (task == null) break;
                    
                    long opStart = System.nanoTime();
                    
                    if ("PUT".equals(task[0])) {
                        cache.put((String)task[1], (String)task[2], (Integer)task[3], Duration.ofHours(1));
                    } else if ("GET".equals(task[0])) {
                        cache.get((String)task[1]);
                    }
                    
                    long opElapsed = System.nanoTime() - opStart;
                    localTimes.add(opElapsed);
                }
                
                operationTimes.addAll(localTimes);
            }));
        }
        
        // Wait for all to complete
        for (Future<?> future : futures) {
            try {
                future.get();
            } catch (ExecutionException e) {
                e.printStackTrace();
            }
        }
        
        executor.shutdown();
        
        long elapsed = System.nanoTime() - startTime;
        double elapsedSeconds = elapsed / 1_000_000_000.0;
        
        // Calculate statistics
        double avgOpTime = 0;
        if (!operationTimes.isEmpty()) {
            long sum = 0;
            for (Long time : operationTimes) {
                sum += time;
            }
            avgOpTime = sum / (double) operationTimes.size() / 1_000_000.0; // Convert to ms
        }
        
        double parallelismFactor = elapsedSeconds > 0 ? 
            (avgOpTime * numOperations / 1000.0) / elapsedSeconds : 1;
        
        BenchmarkResult result = new BenchmarkResult();
        result.put("duration", String.format("%.3f", elapsedSeconds));
        result.put("num_workers", numWorkers);
        result.put("total_operations", numOperations);
        result.put("ops_per_second", String.format("%.2f", numOperations / elapsedSeconds));
        result.put("avg_operation_time_ms", String.format("%.3f", avgOpTime));
        result.put("parallelism_factor", String.format("%.2fx", parallelismFactor));
        
        return result;
    }
    
    /**
     * Eviction strategy benchmark
     */
    public BenchmarkResult benchmarkEvictionStrategy(int cacheSize, int totalInsertions) 
            throws InterruptedException {
        
        IntelligentCache<String, String> cache = new IntelligentCache<>(cacheSize);
        
        System.out.printf("\nRunning Eviction Strategy benchmark (cache size: %d, insertions: %d)...\n", 
                         cacheSize, totalInsertions);
        
        long startTime = System.nanoTime();
        
        // Fill cache to capacity with varying priorities
        for (int i = 0; i < cacheSize; i++) {
            cache.put("key_" + i, "value_" + i, i % 10 + 1, Duration.ofHours(1));
        }
        
        // Force evictions by adding more items than capacity
        int evictionsForced = totalInsertions - cacheSize;
        for (int i = cacheSize; i < totalInsertions; i++) {
            cache.put("key_" + i, "value_" + i, 5, Duration.ofHours(1));
        }
        
        long elapsed = System.nanoTime() - startTime;
        double elapsedSeconds = elapsed / 1_000_000_000.0;
        
        // Check which original items were evicted
        int originalItemsRemaining = 0;
        for (int i = 0; i < cacheSize; i++) {
            if (cache.get("key_" + i) != null) {
                originalItemsRemaining++;
            }
        }
        
        int evictedCount = cacheSize - originalItemsRemaining;
        double evictionEfficiency = evictionsForced > 0 ? 
            (double) evictedCount / evictionsForced * 100 : 0;
        
        BenchmarkResult result = new BenchmarkResult();
        result.put("duration", String.format("%.3f", elapsedSeconds));
        result.put("cache_size", cacheSize);
        result.put("total_insertions", totalInsertions);
        result.put("evictions_forced", evictionsForced);
        result.put("evicted_count", evictedCount);
        result.put("ops_per_second", String.format("%.2f", totalInsertions / elapsedSeconds));
        result.put("eviction_efficiency", String.format("%.1f%%", evictionEfficiency));
        
        return result;
    }
    
    /**
     * TTL operations benchmark
     */
    public BenchmarkResult benchmarkTTLOperations(int numItems, int ttlMs) 
            throws InterruptedException {
        
        IntelligentCache<String, String> cache = new IntelligentCache<>(10000);
        
        System.out.printf("\nRunning TTL Operations benchmark (%d items with %dms TTL)...\n", 
                         numItems, ttlMs);
        
        // Part 1: TTL Expiry Test
        long startTime = System.nanoTime();
        
        // Add items with short TTL
        for (int i = 0; i < numItems; i++) {
            cache.put("ttl_key_" + i, "value_" + i, 5, Duration.ofMillis(ttlMs));
        }
        
        // Wait for expiration
        Thread.sleep(ttlMs + 10);
        
        // Check expired items
        int expiredCount = 0;
        for (int i = 0; i < numItems; i++) {
            if (cache.get("ttl_key_" + i) == null) {
                expiredCount++;
            }
        }
        
        long expiryElapsed = System.nanoTime() - startTime;
        double expiryElapsedSeconds = expiryElapsed / 1_000_000_000.0;
        
        // Part 2: TTL Check Performance (with valid items)
        // Clear cache - create new instance since no clear method
        cache = new IntelligentCache<>(10000);
        
        // Add items with long TTL
        for (int i = 0; i < numItems; i++) {
            cache.put("valid_key_" + i, "value_" + i, 5, Duration.ofHours(1));
        }
        
        // Measure time to check all items
        long checkStart = System.nanoTime();
        int validCount = 0;
        for (int i = 0; i < numItems; i++) {
            if (cache.get("valid_key_" + i) != null) {
                validCount++;
            }
        }
        long checkElapsed = System.nanoTime() - checkStart;
        double checkElapsedSeconds = checkElapsed / 1_000_000_000.0;
        
        double expiryRate = numItems > 0 ? (double) expiredCount / numItems * 100 : 0;
        double checkOpsPerSecond = checkElapsedSeconds > 0 ? numItems / checkElapsedSeconds : 0;
        double avgCheckTimeUs = numItems > 0 ? (checkElapsedSeconds * 1_000_000 / numItems) : 0;
        
        BenchmarkResult result = new BenchmarkResult();
        result.put("ttl_expiry_duration", String.format("%.3f", expiryElapsedSeconds));
        result.put("ttl_check_duration", String.format("%.3f", checkElapsedSeconds));
        result.put("num_items", numItems);
        result.put("ttl_ms", ttlMs);
        result.put("expired_count", expiredCount);
        result.put("expiry_rate", String.format("%.1f%%", expiryRate));
        result.put("valid_count", validCount);
        result.put("check_ops_per_second", String.format("%.2f", checkOpsPerSecond));
        result.put("avg_check_time_us", String.format("%.2f", avgCheckTimeUs));
        
        return result;
    }
    
    /**
     * I/O-bound simulation benchmark
     */
    public BenchmarkResult benchmarkIOSimulation(int numWorkers, int durationSeconds) 
            throws InterruptedException {
        
        IntelligentCache<String, String> cache = new IntelligentCache<>(cacheSize);
        
        AtomicBoolean stopFlag = new AtomicBoolean(false);
        AtomicInteger[] operationCounts = new AtomicInteger[numWorkers];
        
        for (int i = 0; i < numWorkers; i++) {
            operationCounts[i] = new AtomicInteger(0);
        }
        
        ExecutorService executor = Executors.newFixedThreadPool(numWorkers);
        List<Future<?>> futures = new ArrayList<>();
        
        System.out.printf("\nRunning I/O Simulation benchmark (%d workers)...\n", numWorkers);
        System.out.println("Simulating database/network delays where threading helps...");
        
        long startTime = System.nanoTime();
        
        // Start workers
        for (int i = 0; i < numWorkers; i++) {
            final int workerId = i;
            futures.add(executor.submit(() -> {
                int count = 0;
                while (!stopFlag.get()) {
                    try {
                        // Simulate database query
                        Thread.sleep(5); // 5ms "database query"
                        
                        String key = String.format("worker_%d_item_%d", workerId, count % 100);
                        String value = String.format("data_%d_%d", count, System.nanoTime());
                        
                        // Cache the result
                        cache.put(key, value, 5, Duration.ofHours(1));
                        
                        // Try to read some other worker's data
                        int otherWorker = (workerId + random.nextInt(numWorkers - 1) + 1) % numWorkers;
                        String otherKey = String.format("worker_%d_item_%d", otherWorker, random.nextInt(100));
                        String cached = cache.get(otherKey);
                        
                        if (cached != null) {
                            // Simulate processing
                            Thread.sleep(1); // 1ms processing
                        }
                        
                        count += 2; // PUT + GET
                        operationCounts[workerId].set(count);
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        break;
                    }
                }
            }));
        }
        
        // Run for specified duration
        Thread.sleep(durationSeconds * 1000);
        stopFlag.set(true);
        
        // Wait for all to complete
        executor.shutdown();
        executor.awaitTermination(10, TimeUnit.SECONDS);
        
        long elapsed = System.nanoTime() - startTime;
        double elapsedSeconds = elapsed / 1_000_000_000.0;
        
        // Calculate statistics
        int totalOperations = 0;
        for (AtomicInteger count : operationCounts) {
            totalOperations += count.get();
        }
        
        double theoreticalSequentialTime = totalOperations * 0.006; // 6ms per op
        double speedup = theoreticalSequentialTime / elapsedSeconds;
        
        BenchmarkResult result = new BenchmarkResult();
        result.put("duration", String.format("%.2f", elapsedSeconds));
        result.put("num_workers", numWorkers);
        result.put("total_operations", totalOperations);
        result.put("ops_per_second", String.format("%.2f", totalOperations / elapsedSeconds));
        result.put("ops_per_worker", totalOperations / numWorkers);
        result.put("theoretical_sequential_time", String.format("%.2f", theoreticalSequentialTime));
        result.put("speedup", String.format("%.2fx", speedup));
        
        return result;
    }
    
    private static String bytesToHex(byte[] bytes) {
        StringBuilder result = new StringBuilder();
        for (byte b : bytes) {
            result.append(String.format("%02x", b));
        }
        return result.toString();
    }
    
    public static void main(String[] args) throws Exception {
        System.out.println("=" + "=".repeat(59));
        System.out.println("Fair Concurrent Benchmark Suite");
        System.out.println("Java Implementation with True Parallelism");
        System.out.println("=" + "=".repeat(59));
        
        FairConcurrentBenchmark benchmark = new FairConcurrentBenchmark(100000);
        Map<String, Object> results = new HashMap<>();
        
        // Test 1: Producer-Consumer Pattern
        System.out.println("\n1. Producer-Consumer Pattern");
        System.out.println("-".repeat(40));
        BenchmarkResult pcResult = benchmark.benchmarkProducerConsumer(50, 50, 5);
        results.put("producer_consumer", pcResult.metrics);
        pcResult.printResults();
        
        // Test 2: Shared Workload
        System.out.println("\n2. Shared Workload (Fair Comparison)");
        System.out.println("-".repeat(40));
        BenchmarkResult swResult = benchmark.benchmarkSharedWorkload(100, 10000);
        results.put("shared_workload", swResult.metrics);
        swResult.printResults();
        
        // Test 3: I/O Simulation
        System.out.println("\n3. I/O-Bound Simulation");
        System.out.println("-".repeat(40));
        BenchmarkResult ioResult = benchmark.benchmarkIOSimulation(100, 5);
        results.put("io_simulation", ioResult.metrics);
        ioResult.printResults();
        
        // Test 4: Eviction Strategy
        System.out.println("\n4. Eviction Strategy");
        System.out.println("-".repeat(40));
        BenchmarkResult evictResult = benchmark.benchmarkEvictionStrategy(100, 200);
        results.put("eviction", evictResult.metrics);
        evictResult.printResults();
        
        // Test 5: TTL Operations
        System.out.println("\n5. TTL Operations");
        System.out.println("-".repeat(40));
        BenchmarkResult ttlResult = benchmark.benchmarkTTLOperations(100, 100);
        results.put("ttl", ttlResult.metrics);
        ttlResult.printResults();
        
        // Save results
        String timestamp = LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyyMMdd_HHmmss"));
        Map<String, Object> output = new HashMap<>();
        output.put("implementation", "Java (Fair Concurrent)");
        output.put("timestamp", timestamp);
        output.put("cache_size", 100000);
        output.put("benchmarks", results);
        
        Map<String, String> notes = new HashMap<>();
        notes.put("parallelism", "Java has true parallelism with real thread contention");
        notes.put("io_benefit", "Threading helps significantly with I/O-bound operations");
        notes.put("comparison", "These metrics are directly comparable across languages");
        output.put("notes", notes);
        
        // Save to JSON file
        String filename = "results/java_fair_concurrent_" + timestamp + ".json";
        Files.createDirectories(Paths.get("results"));
        
        try (FileWriter writer = new FileWriter(filename)) {
            writer.write(toJson(output));
        }
        
        System.out.println("\n" + "=".repeat(60));
        System.out.println("Results saved to: " + filename);
        System.out.println("=".repeat(60));
        
        System.exit(0);
    }
    
    // Simple JSON serialization
    private static String toJson(Object obj) {
        return toJson(obj, 0);
    }
    
    private static String toJson(Object obj, int indent) {
        String indentStr = "  ".repeat(indent);
        String nextIndent = "  ".repeat(indent + 1);
        
        if (obj == null) {
            return "null";
        } else if (obj instanceof String) {
            return "\"" + obj.toString().replace("\"", "\\\"") + "\"";
        } else if (obj instanceof Number || obj instanceof Boolean) {
            return obj.toString();
        } else if (obj instanceof Map) {
            Map<?, ?> map = (Map<?, ?>) obj;
            if (map.isEmpty()) return "{}";
            StringBuilder sb = new StringBuilder("{\n");
            int count = 0;
            for (Map.Entry<?, ?> entry : map.entrySet()) {
                sb.append(nextIndent).append("\"").append(entry.getKey()).append("\": ");
                sb.append(toJson(entry.getValue(), indent + 1));
                if (++count < map.size()) sb.append(",");
                sb.append("\n");
            }
            sb.append(indentStr).append("}");
            return sb.toString();
        } else {
            return "\"" + obj.toString() + "\"";
        }
    }
}