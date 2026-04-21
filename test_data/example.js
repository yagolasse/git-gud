// Example JavaScript code for syntax highlighting tests

const fs = require('fs');
const path = require('path');

class User {
    constructor(name, email) {
        this.name = name;
        this.email = email;
        this.createdAt = new Date();
    }
    
    getProfile() {
        return {
            name: this.name,
            email: this.email,
            memberSince: this.createdAt.toISOString().split('T')[0]
        };
    }
    
    updateEmail(newEmail) {
        if (this.isValidEmail(newEmail)) {
            this.email = newEmail;
            return true;
        }
        return false;
    }
    
    isValidEmail(email) {
        const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return regex.test(email);
    }
}

function calculateStatistics(numbers) {
    if (!Array.isArray(numbers) || numbers.length === 0) {
        throw new Error('Invalid input: expected non-empty array');
    }
    
    const sum = numbers.reduce((acc, num) => acc + num, 0);
    const average = sum / numbers.length;
    
    const sorted = [...numbers].sort((a, b) => a - b);
    const median = sorted.length % 2 === 0
        ? (sorted[sorted.length / 2 - 1] + sorted[sorted.length / 2]) / 2
        : sorted[Math.floor(sorted.length / 2)];
    
    const min = Math.min(...numbers);
    const max = Math.max(...numbers);
    
    return {
        sum,
        average,
        median,
        min,
        max,
        count: numbers.length
    };
}

async function processFiles(directory) {
    try {
        const files = await fs.promises.readdir(directory);
        const results = [];
        
        for (const file of files) {
            const filePath = path.join(directory, file);
            const stats = await fs.promises.stat(filePath);
            
            if (stats.isFile()) {
                const content = await fs.promises.readFile(filePath, 'utf8');
                results.push({
                    name: file,
                    size: stats.size,
                    lines: content.split('\n').length,
                    modified: stats.mtime
                });
            }
        }
        
        return results;
    } catch (error) {
        console.error(`Error processing directory ${directory}:`, error);
        throw error;
    }
}

// Example usage
const user = new User('John Doe', 'john@example.com');
console.log(user.getProfile());

const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const stats = calculateStatistics(numbers);
console.log('Statistics:', stats);

// Export for module usage
module.exports = {
    User,
    calculateStatistics,
    processFiles
};