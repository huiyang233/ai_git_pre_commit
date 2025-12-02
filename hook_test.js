
function safeCode(input) {
    // Fix: use textContent
    document.getElementById('app').textContent = input;
    
    // Fix: remove infinite loop
    if (input == 0) {
        console.log("input is zero");
    }
}
