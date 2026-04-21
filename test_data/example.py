#!/usr/bin/env python3
# Example Python code for syntax highlighting tests

import os
import sys
from typing import Dict, List, Optional

class Calculator:
    """A simple calculator class."""
    
    def __init__(self, initial_value: int = 0):
        self.value = initial_value
    
    def add(self, x: int) -> int:
        """Add a number to the current value."""
        self.value += x
        return self.value
    
    def multiply(self, x: int) -> int:
        """Multiply the current value by a number."""
        self.value *= x
        return self.value
    
    def reset(self) -> None:
        """Reset the calculator to zero."""
        self.value = 0

def fibonacci(n: int) -> List[int]:
    """Generate Fibonacci sequence up to n terms."""
    if n <= 0:
        return []
    elif n == 1:
        return [0]
    
    sequence = [0, 1]
    while len(sequence) < n:
        sequence.append(sequence[-1] + sequence[-2])
    
    return sequence

def process_data(data: Dict[str, List[int]]) -> Dict[str, float]:
    """Process data and calculate averages."""
    results = {}
    
    for key, values in data.items():
        if values:
            avg = sum(values) / len(values)
            results[key] = avg
        else:
            results[key] = 0.0
    
    return results

if __name__ == "__main__":
    calc = Calculator(10)
    print(f"Initial value: {calc.value}")
    
    calc.add(5)
    print(f"After adding 5: {calc.value}")
    
    calc.multiply(2)
    print(f"After multiplying by 2: {calc.value}")
    
    fib = fibonacci(10)
    print(f"First 10 Fibonacci numbers: {fib}")
    
    data = {
        "A": [1, 2, 3, 4, 5],
        "B": [10, 20, 30],
        "C": []
    }
    
    averages = process_data(data)
    print(f"Averages: {averages}")