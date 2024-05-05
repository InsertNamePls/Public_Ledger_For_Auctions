from flask import Flask, render_template, redirect, url_for
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

if __name__ == "__main__":
    app.run(debug=True)
