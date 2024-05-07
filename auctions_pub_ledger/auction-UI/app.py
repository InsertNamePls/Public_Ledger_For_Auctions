import json
from flask import Flask, render_template, redirect, url_for, jsonify
from flask_bootstrap import Bootstrap

app = Flask(__name__)
Bootstrap(app)

@app.route('/')
def index():
    return render_template('login.html')

@app.route('/login')
def login():
    return render_template('login.html')

@app.route('/home')
def home():
    return render_template('home.html')

@app.route('/logout')
def logout():
    return redirect(url_for('login'))

@app.route('/api/auctions')
def auctions():
    with open('data/auction_data.json') as file:
        data = json.load(file)
    return jsonify(data)

if __name__ == "__main__":
    app.run(debug=True)
