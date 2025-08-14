import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicLong;
import java.time.Duration;
import java.util.function.Consumer;

// Datenklasse für die Cache-Einträge
class CacheEntry<K, V> {
    private final K key;
    private final V value;
    private final int priority;          // Priorität (0 = niedrigste, Integer.MAX_VALUE = höchste)
    private final long expiration;       // Ablaufzeit (in EpochMilli)
    private volatile long lastAccessed;  // Letzter Zugriff (für LRU)

    public CacheEntry(K key, V value, int priority, Duration ttl) {
        this.key = key;
        this.value = value;
        this.priority = priority;
        this.expiration = System.currentTimeMillis() + ttl.toMillis();
        this.lastAccessed = System.currentTimeMillis();
    }

    public K getKey() {
        return key;
    }

    public V getValue() {
        lastAccessed = System.currentTimeMillis();  // Update beim Lesen (LRU)
        return value;
    }

    public int getPriority() {
        return priority;
    }

    public boolean isExpired() {
        return System.currentTimeMillis() > expiration;
    }

    public void markAccessed() {
        lastAccessed = System.currentTimeMillis();  // Für manuelles Update im LRU-Handling
    }
}

public class IntelligentCache<K, V> {
    private final int maxCapacity;               // Maximale Kapazität
    private final ConcurrentHashMap<K, CacheEntry<K, V>> cache;  // Hauptmappe

    // Prioritäten-Struktur: TreeMap mit TreeMap<Integer, LinkedHashMap<K, CacheEntry>> für LRU-Verwaltung
    private final TreeMap<Integer, LinkedHashMap<K, CacheEntry<K, V>>> priorityMap;

    // Statistiken
    private final AtomicLong hitCount = new AtomicLong(0);
    private final AtomicLong missCount = new AtomicLong(0);

    // Ereignis-Callback-Liste
    private final List<CacheEventListener<K, V>> listeners = new ArrayList<>();

    // Thread-safety und automatische Reinigung
    private final Object lock = new Object();
    private final ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor();

    public IntelligentCache(int maxCapacity) {
        this.maxCapacity = maxCapacity;
        this.cache = new ConcurrentHashMap<>();
        this.priorityMap = new TreeMap<>();  // Aufsteigende Prioritäten (niedrigste zuerst)
        // Thread-sücher automatische Cleanup alle 1 Sekunde
        this.scheduler.scheduleAtFixedRate(this::autoCleanupExpired, 0, 1, TimeUnit.SECONDS);
    }

    public V get(K key) {
        synchronized (lock) {
            CacheEntry<K, V> entry = cache.get(key);
            if (entry == null) {
                missCount.incrementAndGet();
                notifyMiss(key);
                return null;
            }

            if (entry.isExpired()) {
                cache.remove(key);
                priorityMap.get(entry.getPriority()).remove(key);
                notifyEvict(entry);
                missCount.incrementAndGet();
                notifyMiss(key);
                return null;
            }

            hitCount.incrementAndGet();
            notifyHit(entry);    // Hit Event
            markRecentAccess(entry);  // Update LRU in Prioritätssegment
            return entry.getValue();
        }
    }

    private void markRecentAccess(CacheEntry<K, V> entry) {
        int priority = entry.getPriority();
        synchronized (lock) {
            LinkedHashMap<K, CacheEntry<K, V>> priMap = priorityMap.get(priority);
            priMap.remove(entry.getKey());     // Remove und reinsert für LRU-Update
            priMap.put(entry.getKey(), entry); // Zugriffszeit wird nicht automatisch aktualisiert, 
        }                                      // daher wird Entry manuell aktualisiert
    }

    public void put(K key, V value, int priority, Duration ttl) {
        synchronized (lock) {
            removeExisting(key);  // Falls Key bereits existiert, entferne eventuell abgelaufenes

            CacheEntry<K, V> newEntry = new CacheEntry<>(key, value, priority, ttl);
            cache.put(key, newEntry);

            // Prioritätssegment anlegen, falls nötig
            priorityMap.computeIfAbsent(priority, k -> new LinkedHashMap<>()).put(key, newEntry);

            // Kapazität prüfen
            if (cache.size() > maxCapacity) {
                evict();  // Entferne Elemente gemäß Eviction Policy
            }
        }
    }

    // Helper: Entfernt bestehendes Element beim Einfügen eines neuen Elements mit gleichem Key
    private void removeExisting(K key) {
        synchronized (lock) {
            CacheEntry<K, V> existing = cache.get(key);
            if (existing != null) {
                cache.remove(key);
                priorityMap.get(existing.getPriority()).remove(key);
            }
        }
    }

    private void evict() {
        synchronized (lock) {
            while (cache.size() > maxCapacity) {
                // 1. Abgelaufene Elemente zuerst entfernen
                if (evictExpired()) continue;

                // 2. LRU innerhalb der Prioritäten
                if (!evictByPriorityAndLRU()) {
                    throw new IllegalStateException("Cannot evict more entries. Cache is oversized.");
                }
            }
        }
    }

    // Helper: Entferne alle abgelaufenen Elemente
    private boolean evictExpired() {
        synchronized (lock) {
            ArrayList<K> expiredKeys = new ArrayList<>();
            for (Map.Entry<K, CacheEntry<K, V>> entry : cache.entrySet()) {
                if (entry.getValue().isExpired()) {
                    expiredKeys.add(entry.getKey());
                }
            }

            if (expiredKeys.size() > 0) {
                for (K key : expiredKeys) {
                    cache.remove(key);
                    priorityMap.get(cache.get(key).getPriority()).remove(key);
                    notifyEvict(cache.get(key));
                }
                return true;
            }
            return false;
        }
    }

    // Helper: Entferne Least Recently Used gemäß Priorität
    private boolean evictByPriorityAndLRU() {
        synchronized (lock) {
            for (int priority : priorityMap.keySet()) {
                LinkedHashMap<K, CacheEntry<K, V>> priMap = priorityMap.get(priority);
                if (priMap.isEmpty()) continue;

                Map.Entry<K, CacheEntry<K, V>> lruEntry = priMap.entrySet().iterator().next();  // Least recently used first
                priMap.remove(lruEntry.getKey());  // Entferne aus Prioritätssegment
                cache.remove(lruEntry.getKey());   // Entferne aus Hauptmappe
                notifyEvict(lruEntry.getValue());
                return true;
            }
        }
        return false;  // Keine weiteren Elemente zum Evicten
    }

    private void autoCleanupExpired() {
        synchronized (lock) {
            for (K key : getExpiredKeys()) {
                cache.remove(key);
                notifyEvict(cache.remove(key));
            }
        }
    }

    private List<K> getExpiredKeys() {
        synchronized (lock) {
            List<K> expiredKeys = new ArrayList<>();
            for (Map.Entry<K, CacheEntry<K, V>> entry : cache.entrySet()) {
                if (entry.getValue().isExpired()) {
                    expiredKeys.add(entry.getKey());
                }
            }
            return expiredKeys;
        }
    }

    public long getHitCount() {
        return hitCount.get();
    }

    public long getMissCount() {
        return missCount.get();
    }

    public void addListener(CacheEventListener<K, V> listener) {
        synchronized (lock) {
            this.listeners.add(listener);
        }
    }

    private void notifyHit(CacheEntry<K, V> entry) {
        listeners.forEach(l -> l.onHit(entry.getKey(), entry.getValue()));
    }

    private void notifyMiss(K key) {
        listeners.forEach(l -> l.onMiss(key));
    }

    private void notifyEvict(CacheEntry<K, V> entry) {
        listeners.forEach(l -> l.onEvict(entry.getKey(), entry.getValue()));
    }

    public interface CacheEventListener<K, V> {
        void onHit(K key, V value);
        void onMiss(K key);
        void onEvict(K key, V value);
    }

    public static void main(String[] args) {
        IntelligentCache<String, String> cache = new IntelligentCache<>(5);

        // Listener hinzufügen
        cache.addListener(new CacheEventListener<String, String>() {
            public void onHit(String key, String value) {
                System.out.println("Hit: " + key);
            }
            public void onMiss(String key) {
                System.out.println("Miss: " + key);
            }
            public void onEvict(String key, String value) {
                System.out.println("Evict: " + key);
            }
        });

        // Prioritäten: 0=niedrig, 1=mittel, 2=höchst
        for (int i = 0; i < 5; i++) {
            cache.put("key" + i, "value" + i, 0, Duration.ofSeconds(10));  // Priorität 0 (niedrigster)
        }

        for (int i = 5; i < 10; i++) {
            cache.put("key" + i, "value" + i, 2, Duration.ofSeconds(10));  // Priorität 2 (höchster)
        }

        // Zugriff zur Aktualisierung des LRUs
        System.out.println(cache.get("key1"));  // Hit: key1
        System.out.println(cache.get("key2"));  // Hit: key2
        System.out.println(cache.get("key9"));  // Zugriff auf hohe Priorität
        System.out.println(cache.get("key15")); // Miss

        // Warte auf Ablauf
        try {
            Thread.sleep(11000);
        } catch (InterruptedException e) {
            e.printStackTrace();
        }

        System.out.println(cache.get("key0"));  // Veraltet → Miss
    }
}