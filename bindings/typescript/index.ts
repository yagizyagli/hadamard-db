// Import the low-level neon binary bridge compilation module
const addon = require('./index.node');

export interface QuantumDataRecord {
    [key: string]: string;
}

export class HadamardTypeScriptClient {
    private nativeEngineBox: any;

    /**
     * Initializes a hardened quantum database instance cluster inside the Node.js memory space.
     * @param shardCapacity Max byte depth threshold before structural sharding occurs. Default 1MB.
     * @param maxQubits Maximum width register capability for cloud QPU dispatch targets. Default 29.
     */
    constructor(shardCapacity: number = 1048576, maxQubits: number = 29) {
        if (shardCapacity <= 0 || maxQubits <= 0) {
            throw new Error("Operational metrics limits must be positive non-zero integers.");
        }
        this.nativeEngineBox = addon.createEngine(shardCapacity, maxQubits);
    }

    /**
     * Ingests JSON records array structures straight to bare-metal storage frames.
     * Operates completely outside the V8 event loop to avoid garbage collection freezes.
     * @param collection Destination catalog workspace name.
     * @param records Complete list of data objects mapping keys to values.
     * @returns A promise resolving to total quantum data shards generated.
     */
    public async insertBulk(collection: string, records: QuantumDataRecord[]): Promise<number> {
        if (!collection || records.length === 0) {
            throw new Error("Collection path register and records buffer queue cannot be null.");
        }
        return addon.loadDataset(this.nativeEngineBox, collection, records);
    }

    /**
     * Translates classical SQL grammar into matrix primitives, accelerates search paths,
     * and streams results back into typed JavaScript array sets.
     * @param sqlStatement Strict relational pseudo-SQL query string.
     * @returns Array matching high-fidelity quantum measurement criteria.
     */
    public async executeQuantumQuery(sqlStatement: string): Promise<QuantumDataRecord[]> {
        if (!sqlStatement) {
            throw new Error("Target statement sequence string execution buffer cannot be blank.");
        }
        return addon.executeQuery(this.nativeEngineBox, sqlStatement);
    }
}
